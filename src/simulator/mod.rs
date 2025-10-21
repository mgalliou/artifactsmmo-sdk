use crate::{
    character::HasCharacterData, damage_type::DamageType, entity::{SimulationCharacter, SimulationEntity, SimulationMonster}, hit::Hit, skill::Skill, CharacterClient, Gear, Slot
};
use artifactsmmo_openapi::models::{FightResult, MonsterSchema, SimpleEffectSchema};
use itertools::Itertools;
use std::{cell::RefCell, cmp::max, rc::Rc};
use strum::IntoEnumIterator;

pub mod damage_type;
pub mod effect_code;
pub mod entity;
pub mod hit;
pub mod has_effects;

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
        attack_dmg: i32,
        dmg_increase: i32,
        critical_strike: i32,
        target_res: i32,
    ) -> f32 {
        let multiplier = Self::average_multiplier(dmg_increase, critical_strike, target_res);

        attack_dmg as f32 * multiplier
    }

    fn average_multiplier(dmg_increase: i32, critical_strike: i32, target_res: i32) -> f32 {
        Self::critless_multiplier(dmg_increase, target_res)
            * (1.0 + critical_strike as f32 * 0.01 * CRIT_MULTIPLIER)
    }

    fn critless_multiplier(dmg_increase: i32, target_res: i32) -> f32 {
        Self::dmg_multiplier(dmg_increase) * Self::res_multiplier(target_res)
    }

    fn crit_multiplier(dmg_increase: i32, target_res: i32) -> f32 {
        Self::critless_multiplier(dmg_increase, target_res) * (1.0 + CRIT_MULTIPLIER)
    }

    fn dmg_multiplier(dmg_increase: i32) -> f32 {
        1.0 + dmg_increase as f32 * 0.01
    }

    fn res_multiplier(target_res: i32) -> f32 {
        let target_res = if target_res > 100 {
            100.0
        } else {
            target_res as f32
        };
        1.0 - target_res * 0.01
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

    pub fn averaged(mut self) -> Self {
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
