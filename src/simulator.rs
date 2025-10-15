use artifactsmmo_openapi::models::{FightResult, ItemSchema, MonsterSchema, SimpleEffectSchema};
use itertools::Itertools;
use std::{cell::RefCell, cmp::max, rc::Rc, sync::Arc};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIs, EnumIter, EnumString};

use crate::{CharacterClient, Gear, Slot, character::HasCharacterData, skill::Skill};

const BASE_HP: u32 = 115;
const HP_PER_LEVEL: u32 = 5;
const BASE_INITIATIVE: i32 = 100;

const MAX_TURN: u32 = 100;
const SECOND_PER_TURN: u32 = 2;
const MIN_FIGHT_CD: u32 = 5;

const REST_HP_PER_SEC: u32 = 5;

const CRIT_MULTIPLIER: f32 = 0.5;
const BURN_MULTIPLIER: f32 = 0.90;

pub struct Simulator {}

impl Simulator {
    pub fn fight(level: u32, gear: &Gear, monster: &MonsterSchema, params: FightParams) -> Fight {
        let char = Rc::new(RefCell::new(SimulationCharacter::new(
            level,
            gear,
            params.utility1_quantity,
            params.utility2_quantity,
            params.missing_hp,
            params.averaged,
        )));
        let monster = Rc::new(RefCell::new(SimulationMonster::new(
            monster,
            params.averaged,
        )));
        let mut fighters: Vec<Rc<RefCell<dyn SimulationEntity>>> =
            vec![char.clone(), monster.clone()];
        fighters.sort_by_key(|e| e.borrow().initiative());
        fighters.reverse();
        let mut fighters_iter = fighters.iter().cycle();
        let mut turn = 1;
        while turn <= MAX_TURN
            && char.borrow().current_health > 0
            && monster.borrow().current_health > 0
            && let Some(fighter) = fighters_iter.next()
        {
            if fighter.borrow().is_monster() {
                monster
                    .borrow_mut()
                    .turn_against(&mut *char.borrow_mut(), turn);
            } else {
                char.borrow_mut()
                    .turn_against(&mut *monster.borrow_mut(), turn);
            }
            turn += 1;
        }
        let char = char.borrow();
        let monster = monster.borrow();
        Fight {
            turns: turn,
            hp: char.current_health,
            monster_hp: monster.current_health,
            hp_lost: char.starting_hp - char.current_health,
            result: if char.current_health <= 0 || turn > MAX_TURN {
                FightResult::Loss
            } else {
                FightResult::Win
            },
            cd: Self::fight_cd(gear.haste(), turn),
        }
    }

    /// Compute the average damage an attack will do against the given `target_resistance`.
    pub fn average_dmg(
        attack_damage: i32,
        damage_increase: i32,
        critical_strike: i32,
        target_resistance: i32,
    ) -> f32 {
        let multiplier =
            Self::average_multiplier(damage_increase, critical_strike, target_resistance);

        attack_damage as f32 * multiplier
    }

    fn average_multiplier(
        damage_increase: i32,
        critical_strike: i32,
        target_resistance: i32,
    ) -> f32 {
        Self::critless_multiplier(damage_increase, target_resistance)
            * (1.0 + critical_strike as f32 * 0.01 * CRIT_MULTIPLIER)
    }

    fn critless_multiplier(damage_increase: i32, target_resistance: i32) -> f32 {
        Self::dmg_multiplier(damage_increase) * Self::res_multiplier(target_resistance)
    }

    fn crit_multiplier(damage_increase: i32, target_resistance: i32) -> f32 {
        Self::critless_multiplier(damage_increase, target_resistance) * (1.0 + CRIT_MULTIPLIER)
    }

    fn dmg_multiplier(dmg_increase: i32) -> f32 {
        1.0 + dmg_increase as f32 * 0.01
    }

    fn res_multiplier(target_resistance: i32) -> f32 {
        let resistance = if target_resistance > 100 {
            100.0
        } else {
            target_resistance as f32
        };
        1.0 - resistance * 0.01
    }

    pub fn time_to_rest(health: u32) -> u32 {
        health / REST_HP_PER_SEC
            + if health.is_multiple_of(REST_HP_PER_SEC) {
                0
            } else {
                1
            }
    }

    fn fight_cd(haste: i32, turns: u32) -> u32 {
        max(
            MIN_FIGHT_CD,
            ((turns * SECOND_PER_TURN) as f32
                - (haste as f32 * 0.01) * (turns * SECOND_PER_TURN) as f32)
                .round() as u32,
        )
    }

    pub fn gather_cd(resource_level: u32, cooldown_reduction: i32) -> u32 {
        let level = resource_level as f32;
        let reduction = cooldown_reduction as f32;

        ((30.0 + (level / 2.0)) * (1.0 + reduction * 0.01)).round() as u32
    }
}

pub struct FightParams {
    utility1_quantity: u32,
    utility2_quantity: u32,
    missing_hp: i32,
    averaged: bool,
    ignore_death: bool,
}

impl FightParams {
    pub fn ignore_death(mut self) -> Self {
        self.ignore_death = true;
        self
    }

    pub fn average(mut self) -> Self {
        self.averaged = true;
        self
    }
}

impl From<&CharacterClient> for FightParams {
    fn from(value: &CharacterClient) -> Self {
        Self {
            utility1_quantity: value.quantity_in_slot(Slot::Utility1),
            utility2_quantity: value.quantity_in_slot(Slot::Utility2),
            missing_hp: value.missing_hp(),
            ..Default::default()
        }
    }
}

impl Default for FightParams {
    fn default() -> Self {
        Self {
            utility1_quantity: 100,
            utility2_quantity: 100,
            missing_hp: 0,
            averaged: false,
            ignore_death: false,
        }
    }
}

trait SimulationEntity: HasEffects {
    fn turn_against(&mut self, enemy: &mut dyn SimulationEntity, turn: u32) {
        if turn == self.reconstitution() as u32 {
            self.set_health(self.max_hp());
        }
        if self.current_health() < self.max_hp() / 2 {
            self.consume_restore_utilities();
        }
        if self.current_turn().is_multiple_of(3) {
            self.receive_healing();
        }
        if self.burning() > 0 {
            self.suffer_burning();
            if self.current_health() < 1 {
                return;
            }
        }
        if self.poisoned() > 0 {
            self.suffer_poisoning();
            if self.current_health() < 1 {
                return;
            }
        }
        if self.current_turn() == 1 {
            self.apply_burn(enemy);
            self.apply_poison(enemy);
        }
        for hit in self.hits_against(enemy, self.average()).iter() {
            enemy.dec_health(hit.damage);
            if hit.is_crit {
                self.inc_health(hit.damage * self.lifesteal() / 100);
            }
            if enemy.current_health() < 1 {
                return;
            }
            if enemy.corrupted() > 0 {
                enemy.suffer_corruption(hit.r#type);
            }
        }
        self.inc_turn();
    }

    fn apply_burn(&self, enemy: &mut dyn SimulationEntity) {
        enemy.set_burning(self.critless_damage_against(enemy) * self.burn() / 100);
    }

    fn apply_poison(&self, enemy: &mut dyn SimulationEntity) {
        enemy.set_poisoned(self.poison());
    }

    fn receive_healing(&mut self) {
        self.inc_health((self.max_hp() as f32 * self.healing() as f32 * 0.01).round() as i32)
    }

    fn consume_restore_utilities(&mut self) {
        if let Some(utility1) = self.utility1()
            && self.utility1_quantity() > 0
        {
            let restore = utility1.restore();
            if restore > 0 {
                self.inc_health(restore);
                self.dec_utility1();
            }
        }
        if let Some(utility2) = self.utility2()
            && self.utility2_quantity() > 0
        {
            let restore = utility2.restore();
            if restore > 0 {
                self.inc_health(restore);
                self.dec_utility2();
            }
        }
    }

    fn suffer_burning(&mut self) {
        self.set_burning((self.burning() as f32 * BURN_MULTIPLIER).round() as i32);
        self.dec_health(self.burning());
    }

    fn suffer_poisoning(&mut self) {
        self.dec_health(self.poisoned());
    }

    fn average(&self) -> bool;
    fn current_turn(&self) -> u32;
    fn inc_turn(&mut self);
    fn max_hp(&self) -> i32;
    fn current_health(&self) -> i32;
    fn set_health(&mut self, value: i32);
    fn burning(&self) -> i32;
    fn set_burning(&mut self, value: i32);
    fn poisoned(&self) -> i32;
    fn set_poisoned(&mut self, value: i32);
    fn suffer_corruption(&mut self, r#type: DamageType);

    fn inc_health(&mut self, value: i32) {
        let missing = self.max_hp() - self.current_health();
        let value = if value > missing { missing } else { value };
        self.set_health(self.current_health() + value)
    }

    fn dec_health(&mut self, value: i32) {
        self.set_health(self.current_health() - value)
    }

    fn utility1(&self) -> Option<Arc<ItemSchema>> {
        None
    }

    fn utility2(&self) -> Option<Arc<ItemSchema>> {
        None
    }

    fn utility1_quantity(&self) -> u32 {
        0
    }

    fn utility2_quantity(&self) -> u32 {
        0
    }

    fn dec_utility1(&mut self) {}
    fn dec_utility2(&mut self) {}
    fn is_monster(&self) -> bool;
}

pub struct SimulationCharacter<'a> {
    average: bool,
    gear: &'a Gear,
    starting_hp: i32,
    max_hp: i32,
    inititive: i32,

    current_turn: u32,
    current_health: i32,

    fire_res: i32,
    earth_res: i32,
    water_res: i32,
    air_res: i32,

    burning: i32,
    poisoned: i32,

    utility1_quantity: u32,
    utility2_quantity: u32,
}

impl<'a> SimulationCharacter<'a> {
    fn new(
        level: u32,
        gear: &'a Gear,
        utility1_quantity: u32,
        utility2_quantity: u32,
        missing_hp: i32,
        average: bool,
    ) -> Self {
        let base_hp = (BASE_HP + HP_PER_LEVEL * level) as i32;
        let max_hp = base_hp + gear.health();
        let starting_hp = max_hp - missing_hp;
        Self {
            gear,
            max_hp,
            starting_hp,
            inititive: BASE_INITIATIVE + gear.initiative(),
            current_health: starting_hp,
            current_turn: 1,
            fire_res: gear.resistance(DamageType::Fire),
            earth_res: gear.resistance(DamageType::Earth),
            water_res: gear.resistance(DamageType::Water),
            air_res: gear.resistance(DamageType::Air),
            utility1_quantity,
            utility2_quantity,
            burning: 0,
            poisoned: 0,
            average,
        }
    }
}

impl<'a> SimulationEntity for SimulationCharacter<'a> {
    fn current_turn(&self) -> u32 {
        self.current_turn
    }

    fn inc_turn(&mut self) {
        self.current_turn += 1
    }

    fn max_hp(&self) -> i32 {
        self.max_hp
    }

    fn current_health(&self) -> i32 {
        self.current_health
    }

    fn poisoned(&self) -> i32 {
        self.poisoned
    }

    fn burning(&self) -> i32 {
        self.burning
    }

    fn set_burning(&mut self, value: i32) {
        self.burning = value;
    }

    fn set_poisoned(&mut self, value: i32) {
        self.poisoned = value;
    }

    fn set_health(&mut self, value: i32) {
        self.current_health = value;
    }

    fn average(&self) -> bool {
        self.average
    }

    fn utility1(&self) -> Option<Arc<ItemSchema>> {
        self.gear.utility1.clone()
    }

    fn utility2(&self) -> Option<Arc<ItemSchema>> {
        self.gear.utility2.clone()
    }

    fn utility1_quantity(&self) -> u32 {
        self.utility1_quantity
    }

    fn utility2_quantity(&self) -> u32 {
        self.utility2_quantity
    }

    fn dec_utility1(&mut self) {
        self.utility1_quantity = self.utility1_quantity().saturating_sub(1);
    }

    fn dec_utility2(&mut self) {
        self.utility2_quantity = self.utility2_quantity().saturating_sub(1);
    }

    fn suffer_corruption(&mut self, r#type: DamageType) {
        let corrupted = self.corrupted();
        match r#type {
            DamageType::Fire => self.fire_res -= corrupted,
            DamageType::Earth => self.earth_res -= corrupted,
            DamageType::Water => self.water_res -= corrupted,
            DamageType::Air => self.air_res -= corrupted,
        }
    }

    fn is_monster(&self) -> bool {
        false
    }
}

impl<'a> HasEffects for SimulationCharacter<'a> {
    fn resistance(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.fire_res,
            DamageType::Earth => self.earth_res,
            DamageType::Water => self.water_res,
            DamageType::Air => self.air_res,
        }
    }

    fn initiative(&self) -> i32 {
        self.inititive
    }

    fn effect_value(&self, effect: &str) -> i32 {
        self.gear.effect_value(effect)
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema> {
        self.gear.effects()
    }
}

pub struct SimulationMonster<'a> {
    average: bool,
    monster: &'a MonsterSchema,

    current_turn: u32,
    current_health: i32,

    fire_res: i32,
    earth_res: i32,
    water_res: i32,
    air_res: i32,

    burning: i32,
    poisoned: i32,
}

impl<'a> SimulationMonster<'a> {
    fn new(monster: &'a MonsterSchema, average: bool) -> Self {
        Self {
            monster,
            current_health: monster.health(),
            current_turn: 1,
            burning: 0,
            poisoned: 0,
            average,
            fire_res: monster.resistance(DamageType::Fire),
            earth_res: monster.resistance(DamageType::Earth),
            water_res: monster.resistance(DamageType::Water),
            air_res: monster.resistance(DamageType::Air),
        }
    }
}

impl<'a> SimulationEntity for SimulationMonster<'a> {
    fn current_turn(&self) -> u32 {
        self.current_turn
    }

    fn inc_turn(&mut self) {
        self.current_turn += 1
    }

    fn max_hp(&self) -> i32 {
        self.monster.hp
    }

    fn current_health(&self) -> i32 {
        self.current_health
    }

    fn poisoned(&self) -> i32 {
        self.poisoned
    }

    fn burning(&self) -> i32 {
        self.burning
    }

    fn set_burning(&mut self, value: i32) {
        self.burning = value
    }

    fn set_poisoned(&mut self, value: i32) {
        self.poisoned = value
    }

    fn set_health(&mut self, value: i32) {
        self.current_health = value;
    }

    fn average(&self) -> bool {
        self.average
    }

    fn suffer_corruption(&mut self, r#type: DamageType) {
        let corrupted = self.corrupted();
        match r#type {
            DamageType::Fire => self.fire_res -= corrupted,
            DamageType::Earth => self.earth_res -= corrupted,
            DamageType::Water => self.water_res -= corrupted,
            DamageType::Air => self.air_res -= corrupted,
        }
    }

    fn is_monster(&self) -> bool {
        true
    }
}

impl<'a> HasEffects for SimulationMonster<'a> {
    fn health(&self) -> i32 {
        self.monster.health()
    }

    fn attack_damage(&self, r#type: DamageType) -> i32 {
        self.monster.attack_damage(r#type)
    }

    fn critical_strike(&self) -> i32 {
        self.monster.critical_strike()
    }

    fn resistance(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.fire_res,
            DamageType::Earth => self.earth_res,
            DamageType::Water => self.water_res,
            DamageType::Air => self.air_res,
        }
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema> {
        self.monster.effects()
    }
}

#[derive(Debug)]
pub struct Fight {
    pub turns: u32,
    pub hp: i32,
    pub monster_hp: i32,
    pub hp_lost: i32,
    pub result: FightResult,
    pub cd: u32,
}

impl Fight {
    pub fn is_winning(&self) -> bool {
        matches!(self.result, FightResult::Win)
    }

    pub fn is_losing(&self) -> bool {
        matches!(self.result, FightResult::Loss)
    }
}

pub struct Hit {
    pub r#type: DamageType,
    pub damage: i32,
    pub is_crit: bool,
}

impl Hit {
    pub fn new(
        attack_damage: i32,
        damage_increase: i32,
        r#type: DamageType,
        target_resistance: i32,
        is_crit: bool,
    ) -> Hit {
        let mut damage = attack_damage as f32;

        damage *= if is_crit {
            Simulator::crit_multiplier(damage_increase, target_resistance)
        } else {
            Simulator::critless_multiplier(damage_increase, target_resistance)
        };
        Hit {
            r#type,
            damage: damage.round() as i32,
            is_crit,
        }
    }

    pub fn average(
        attack_damage: i32,
        damage_increase: i32,
        critical_strike: i32,
        r#type: DamageType,
        target_resistance: i32,
    ) -> Hit {
        let mut damage = attack_damage as f32;

        damage *=
            Simulator::average_multiplier(damage_increase, critical_strike, target_resistance);
        Hit {
            r#type,
            damage: damage.round() as i32,
            is_crit: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumIter, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DamageType {
    Fire,
    Earth,
    Water,
    Air,
}

impl DamageType {
    pub fn into_attack(&self) -> &'static str {
        match self {
            DamageType::Fire => "attack_fire",
            DamageType::Earth => "attack_earth",
            DamageType::Water => "attack_water",
            DamageType::Air => "attack_air",
        }
    }

    pub fn into_damage(&self) -> &'static str {
        match self {
            DamageType::Fire => "dmg_fire",
            DamageType::Earth => "dmg_earth",
            DamageType::Water => "dmg_water",
            DamageType::Air => "dmg_air",
        }
    }

    pub fn into_boost_damage(&self) -> &'static str {
        match self {
            DamageType::Fire => "boost_dmg_fire",
            DamageType::Earth => "boost_dmg_earth",
            DamageType::Water => "boost_dmg_water",
            DamageType::Air => "boost_dmg_air",
        }
    }

    pub fn into_resistance(&self) -> &'static str {
        match self {
            DamageType::Fire => "res_fire",
            DamageType::Earth => "res_earth",
            DamageType::Water => "res_water",
            DamageType::Air => "res_air",
        }
    }
}

const HP: &str = "hp";
const BOOST_HP: &str = "boost_hp";
const HEAL: &str = "heal";
const HEALING: &str = "healing";
const RESTORE: &str = "restore";
const HASTE: &str = "haste";
const DMG: &str = "dmg";
const CRITICAL_STRIKE: &str = "critical_strike";
const POISON: &str = "poison";
const WISDOM: &str = "wisdom";
const LIFESTEAL: &str = "lifesteal";
const BURN: &str = "burn";
const RECONSTITUTION: &str = "reconstitution";
const CORRUPTED: &str = "corrupted";
const PROSPECTING: &str = "prospecting";
const INVENTORY_SPACE: &str = "inventory_space";
const INITIATIVE: &str = "initiative";
const THREAT: &str = "threat";

pub trait HasEffects {
    fn health(&self) -> i32 {
        self.effect_value(HP) + self.effect_value(BOOST_HP)
    }

    fn heal(&self) -> i32 {
        self.effect_value(HEAL)
    }

    fn healing(&self) -> i32 {
        self.effect_value(HEALING)
    }

    fn restore(&self) -> i32 {
        self.effect_value(RESTORE)
    }

    fn haste(&self) -> i32 {
        self.effect_value(HASTE)
    }

    fn initiative(&self) -> i32 {
        self.effect_value(INITIATIVE)
    }

    fn threat(&self) -> i32 {
        self.effect_value(THREAT)
    }

    fn attack_damage(&self, r#type: DamageType) -> i32 {
        self.effect_value(r#type.into_attack())
    }

    fn damage_increase(&self, r#type: DamageType) -> i32 {
        self.effect_value(DMG)
            + self.effect_value(r#type.into_damage())
            + self.effect_value(r#type.into_boost_damage())
    }

    fn resistance(&self, r#type: DamageType) -> i32 {
        self.effect_value(r#type.into_resistance())
    }

    fn critical_strike(&self) -> i32 {
        self.effect_value(CRITICAL_STRIKE)
    }

    fn poison(&self) -> i32 {
        self.effect_value(POISON)
    }

    fn lifesteal(&self) -> i32 {
        self.effect_value(LIFESTEAL)
    }

    fn burn(&self) -> i32 {
        self.effect_value(BURN)
    }

    fn reconstitution(&self) -> i32 {
        self.effect_value(RECONSTITUTION)
    }

    fn corrupted(&self) -> i32 {
        self.effect_value(CORRUPTED)
    }

    fn wisdom(&self) -> i32 {
        self.effect_value(WISDOM)
    }

    fn prospecting(&self) -> i32 {
        self.effect_value(PROSPECTING)
    }

    fn skill_cooldown_reduction(&self, skill: Skill) -> i32 {
        self.effect_value(skill.as_ref())
    }

    fn inventory_space(&self) -> i32 {
        self.effect_value(INVENTORY_SPACE)
    }

    fn effect_value(&self, effect: &str) -> i32 {
        self.effects()
            .iter()
            .find_map(|e| (e.code == effect).then_some(e.value))
            .unwrap_or(0)
    }

    fn hits_against(&self, enemy: &dyn HasEffects, average: bool) -> Vec<Hit> {
        let mut is_crit = false;
        if !average {
            is_crit = rand::random_range(0..=100) <= self.critical_strike();
        }
        DamageType::iter()
            .filter_map(|t| {
                let attack_damage = self.attack_damage(t);
                (attack_damage > 0).then_some(if average {
                    Hit::average(
                        self.attack_damage(t),
                        self.damage_increase(t),
                        self.critical_strike(),
                        t,
                        enemy.resistance(t),
                    )
                } else {
                    Hit::new(
                        self.attack_damage(t),
                        self.damage_increase(t),
                        t,
                        enemy.resistance(t),
                        is_crit,
                    )
                })
            })
            .collect_vec()
    }

    fn critless_damage_against(&self, enemy: &dyn HasEffects) -> i32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    self.attack_damage(t),
                    self.damage_increase(t),
                    0,
                    enemy.resistance(t),
                )
                .round() as i32
            })
            .sum()
    }

    // Returns the damage boost provided by the `boost` entity to the `self` entity against the
    // `target` entity
    fn average_dmg_boost_against_with(
        &self,
        boost: &dyn HasEffects,
        target: &dyn HasEffects,
    ) -> f32 {
        self.average_dmg_against_with(boost, target) - self.average_dmg_against(target)
    }

    fn average_dmg_reduction_against(&self, target: &dyn HasEffects) -> f32
    where
        Self: Sized,
    {
        target.average_damage() - target.average_dmg_against(self)
    }

    fn average_dmg_against(&self, target: &dyn HasEffects) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    self.attack_damage(t),
                    0,
                    self.critical_strike(),
                    target.resistance(t),
                )
            })
            .sum()
    }

    // Returns the average attack damage done by the `self` entity against the `target ` entity with additionnnal effects from the `boost` entity
    // damage `boost`
    fn average_dmg_against_with(&self, boost: &dyn HasEffects, target: &dyn HasEffects) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    self.attack_damage(t),
                    boost.damage_increase(t),
                    self.critical_strike() + boost.critical_strike(),
                    target.resistance(t),
                )
            })
            .sum()
    }

    fn average_damage(&self) -> f32 {
        DamageType::iter()
            .map(|t| Simulator::average_dmg(self.attack_damage(t), 0, self.critical_strike(), 0))
            .sum()
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema>;
}

#[derive(Debug, Copy, Clone, PartialEq, Display, AsRefStr, EnumIter, EnumString, EnumIs)]
#[strum(serialize_all = "snake_case")]
pub enum EffectType {
    CriticalStrike,
    Burn,
    Poison,
    Haste,
    Prospecting,
    Wisdom,
    Restore,
    Hp,
    BoostHp,
    Heal,
    Healing,
    Lifesteal,
    InventorySpace,

    AttackFire,
    AttackEarth,
    AttackWater,
    AttackAir,

    Dmg,
    DmgFire,
    DmgEarth,
    DmgWater,
    DmgAir,

    BoostDmgFire,
    BoostDmgEarth,
    BoostDmgWater,
    BoostDmgAir,
    ResDmgFire,
    ResDmgEarth,
    ResDmgWater,
    ResDmgAir,

    Mining,
    Woodcutting,
    Fishing,
    Alchemy,

    //Monster specific
    Reconstitution,
    Corrupted,
}

impl PartialEq<EffectType> for String {
    fn eq(&self, other: &EffectType) -> bool {
        other.as_ref() == *self
    }
}

#[cfg(test)]
mod tests {
    use crate::Simulator;

    //TODO: rewrite tests
    // use crate::{ITEMS, MONSTERS};
    //
    // use super::*;
    //
    // #[test]
    // fn gather() {
    //     assert_eq!(Simulator::gather(17, 1, -10,), 21);
    // }
    //
    // #[test]
    // fn kill_deathnight() {
    //     let gear = Gear {
    //         weapon: ITEMS.get("skull_staff"),
    //         shield: ITEMS.get("steel_shield"),
    //         helmet: ITEMS.get("piggy_helmet"),
    //         body_armor: ITEMS.get("bandit_armor"),
    //         leg_armor: ITEMS.get("piggy_pants"),
    //         boots: ITEMS.get("adventurer_boots"),
    //         ring1: ITEMS.get("skull_ring"),
    //         ring2: ITEMS.get("skull_ring"),
    //         amulet: ITEMS.get("ruby_amulet"),
    //         artifact1: None,
    //         artifact2: None,
    //         artifact3: None,
    //         utility1: None,
    //         utility2: None,
    //     };
    //     let fight = Simulator::fight(30, 0, &gear, &MONSTERS.get("death_knight").unwrap(), false);
    //     println!("{:?}", fight);
    //     assert_eq!(fight.result, FightResult::Win);
    // }
    //
    // #[test]
    // fn kill_cultist_emperor() {
    //     let gear = Gear {
    //         weapon: ITEMS.get("magic_bow"),
    //         shield: ITEMS.get("gold_shield"),
    //         helmet: ITEMS.get("strangold_helmet"),
    //         body_armor: ITEMS.get("serpent_skin_armor"),
    //         leg_armor: ITEMS.get("strangold_legs_armor"),
    //         boots: ITEMS.get("gold_boots"),
    //         ring1: ITEMS.get("emerald_ring"),
    //         ring2: ITEMS.get("emerald_ring"),
    //         amulet: ITEMS.get("ancestral_talisman"),
    //         artifact1: ITEMS.get("christmas_star"),
    //         artifact2: None,
    //         artifact3: None,
    //         utility1: None,
    //         utility2: None,
    //     };
    //     let fight = Simulator::fight(
    //         40,
    //         0,
    //         &gear,
    //         &MONSTERS.get("cultist_emperor").unwrap(),
    //         false,
    //     );
    //     println!("{:?}", fight);
    //     assert_eq!(fight.result, FightResult::Win);
    // }
    #[test]
    fn check_gather_cd() {
        assert_eq!(Simulator::gather_cd(1, -10), 27)
    }
}
