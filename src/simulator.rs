use crate::{char::Skill, gear::Gear, items::DamageType};
use artifactsmmo_openapi::models::{FightResult, MonsterSchema, SimpleEffectSchema};
use std::cmp::max;

const BASE_HP: u32 = 115;
const MAX_TURN: u32 = 100;
const HP_PER_LEVEL: u32 = 5;
const CRIT_MULTIPLIER: f32 = 1.5;
const BURN_MULTIPLIER: f32 = 0.90;

pub struct Simulator {}

impl Simulator {
    pub fn average_fight(
        level: u32,
        missing_hp: i32,
        gear: &Gear,
        monster: &MonsterSchema,
        ignore_death: bool,
    ) -> Fight {
        let base_hp = (BASE_HP + HP_PER_LEVEL * level) as i32;
        let starting_hp = base_hp + gear.health() - missing_hp;
        let mut hp = starting_hp;
        let mut monster_hp = monster.health();
        let mut turn = 1;
        let mut burn = gear.critless_damage_against(monster) * gear.burn() / 100;
        let mut monster_burn = gear.critless_damage_from(monster) * monster.burn() / 100;

        loop {
            //character turn
            if turn % 2 == 1 {
                if turn > 1 {
                    if monster_burn > 0 {
                        Self::decrease_burn(&mut monster_burn);
                        hp -= monster_burn;
                        if hp <= 0 && !ignore_death {
                            break;
                        }
                    }
                    hp -= monster.poison();
                    if hp <= 0 && !ignore_death {
                        break;
                    }
                }
                let damage = gear.average_damage_against(monster);
                monster_hp -= damage;
                hp += damage * gear.lifesteal() / 100;
                if monster_hp <= 0 {
                    break;
                }
            //monster turn
            } else {
                if turn == monster.reconstitution() as u32 {
                    monster_hp = monster.health();
                }
                if burn > 0 {
                    Self::decrease_burn(&mut burn);
                    monster_hp -= burn;
                    if monster_hp <= 0 {
                        break;
                    }
                }
                monster_hp -= gear.poison();
                if monster_hp <= 0 {
                    break;
                }
                if hp < (base_hp + gear.health()) / 2 {
                    hp += gear.utility1.as_ref().map(|u| u.restore()).unwrap_or(0);
                    hp += gear.utility2.as_ref().map(|u| u.restore()).unwrap_or(0);
                }
                let damage = gear.avarage_damage_from(monster);
                hp -= damage;
                monster_hp += damage * gear.lifesteal() / 100;
                if hp <= 0 && !ignore_death {
                    break;
                }
            }
            if turn >= MAX_TURN {
                break;
            }
            turn += 1;
        }
        Fight {
            turns: turn,
            hp,
            monster_hp,
            hp_lost: starting_hp - hp,
            result: if hp <= 0 || turn > MAX_TURN {
                FightResult::Loss
            } else {
                FightResult::Win
            },
            cd: Self::fight_cd(gear.haste(), turn),
        }
    }

    pub fn random_fight(
        level: u32,
        missing_hp: i32,
        gear: &Gear,
        monster: &MonsterSchema,
        ignore_death: bool,
    ) -> Fight {
        let base_hp = (BASE_HP + HP_PER_LEVEL * level) as i32;
        let starting_hp = base_hp + gear.health() - missing_hp;
        let mut hp = starting_hp;
        let mut monster_hp = monster.health();
        let mut turn = 1;
        let mut burn = gear.critless_damage_against(monster) * gear.burn() / 100;
        let mut monster_burn = gear.critless_damage_from(monster) * monster.burn() / 100;

        loop {
            //character turn
            if turn % 2 == 1 {
                if turn > 1 {
                    if monster_burn > 0 {
                        Self::decrease_burn(&mut monster_burn);
                        hp -= monster_burn;
                        if hp <= 0 && !ignore_death {
                            break;
                        }
                    }
                    hp -= monster.poison();
                    if hp <= 0 && !ignore_death {
                        break;
                    }
                }
                for h in gear.simulate_hits_against(monster).iter() {
                    monster_hp -= h.damage;
                    if h.is_crit {
                        hp += h.damage * gear.lifesteal() / 100;
                    }
                    if monster_hp <= 0 {
                        break;
                    }
                }
            //monster turn
            } else {
                if turn == monster.reconstitution() as u32 {
                    monster_hp = monster.health();
                }
                if burn > 0 {
                    Self::decrease_burn(&mut burn);
                    monster_hp -= burn;
                    if monster_hp <= 0 {
                        break;
                    }
                }
                monster_hp -= gear.poison();
                if monster_hp <= 0 {
                    break;
                }
                if hp < (base_hp + gear.health()) / 2 {
                    hp += gear.utility1.as_ref().map_or(0, |u| u.restore());
                    hp += gear.utility2.as_ref().map_or(0, |u| u.restore());
                }
                for h in gear.simulate_hits_from(monster).iter() {
                    hp -= h.damage;
                    if h.is_crit {
                        monster_hp += h.damage * monster.lifesteal() / 100;
                    }
                    if hp <= 0 && !ignore_death {
                        break;
                    }
                }
            }
            if turn >= MAX_TURN {
                break;
            }
            turn += 1;
        }
        Fight {
            turns: turn,
            hp,
            monster_hp,
            hp_lost: starting_hp - hp,
            result: if hp <= 0 || turn > MAX_TURN {
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
        let multiplier = (1.0 + damage_increase as f32 * 0.01)
            * (1.0 + critical_strike as f32 * 0.005)
            * (1.0 - target_resistance as f32 * 0.01);

        attack_damage as f32 * multiplier
    }

    pub fn simulate_hit(
        attack_damage: i32,
        damage_increase: i32,
        critical_strike: i32,
        r#type: DamageType,
        target_resistance: i32,
    ) -> Hit {
        let mut damage = attack_damage as f32;
        let mut is_crit = false;
        let multiplier =
            (1.0 + damage_increase as f32 * 0.01) * (1.0 - target_resistance as f32 * 0.01);

        damage *= multiplier;
        if rand::random_range(0..=100) <= critical_strike {
            damage *= CRIT_MULTIPLIER;
            is_crit = true
        }
        Hit {
            r#type,
            damage: damage.round() as i32,
            is_crit,
        }
    }

    pub fn time_to_rest(health: u32) -> u32 {
        health / 5 + if health % 5 > 0 { 1 } else { 0 }
    }

    fn fight_cd(haste: i32, turns: u32) -> u32 {
        max(
            5,
            ((turns * 2) as f32 - (haste as f32 * 0.01) * (turns * 2) as f32).round() as u32,
        )
    }

    pub fn gather_cd(resource_level: u32, cooldown_reduction: i32) -> u32 {
        ((30.0 + (resource_level as f32 / 2.0)) * (1.0 + cooldown_reduction as f32 * 0.01)) as u32
    }

    fn decrease_burn(burn: &mut i32) {
        *burn = (*burn as f32 * BURN_MULTIPLIER).round() as i32;
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

pub trait HasEffects {
    const HP: &str = "hp";
    const BOOST_HP: &str = "boost_hp";
    const HEAL: &str = "heal";
    const RESTORE: &str = "restore";
    const HASTE: &str = "haste";
    const DMG: &str = "dmg";
    const CRITICAL_STRIKE: &str = "critical_strike";
    const POISON: &str = "poison";
    const WISDOM: &str = "wisdom";
    const LIFESTEAL: &str = "lifesteal";
    const BURN: &str = "burn";
    const RECONSTITUTION: &str = "reconstitution";
    const PROSPECTING: &str = "prospecting";
    const INVENTORY_SPACE: &str = "inventory_space";

    fn health(&self) -> i32 {
        self.effect_value(Self::HP) + self.effect_value(Self::BOOST_HP)
    }

    fn heal(&self) -> i32 {
        self.effect_value(Self::HEAL)
    }

    fn restore(&self) -> i32 {
        self.effect_value(Self::RESTORE)
    }

    fn haste(&self) -> i32 {
        self.effect_value(Self::HASTE)
    }

    fn attack_damage(&self, r#type: DamageType) -> i32 {
        self.effect_value(r#type.into_attack())
    }

    fn damage_increase(&self, r#type: DamageType) -> i32 {
        self.effect_value(Self::DMG)
            + self.effect_value(r#type.into_damage())
            + self.effect_value(r#type.into_boost_damage())
    }

    fn resistance(&self, r#type: DamageType) -> i32 {
        self.effect_value(r#type.into_resistance())
    }

    fn critical_strike(&self) -> i32 {
        self.effect_value(Self::CRITICAL_STRIKE)
    }

    fn poison(&self) -> i32 {
        self.effect_value(Self::POISON)
    }

    fn lifesteal(&self) -> i32 {
        self.effect_value(Self::LIFESTEAL)
    }

    fn burn(&self) -> i32 {
        self.effect_value(Self::BURN)
    }

    fn reconstitution(&self) -> i32 {
        self.effect_value(Self::RECONSTITUTION)
    }

    fn wisdom(&self) -> i32 {
        self.effect_value(Self::WISDOM)
    }

    fn prospecting(&self) -> i32 {
        self.effect_value(Self::PROSPECTING)
    }

    fn skill_cooldown_reduction(&self, skill: Skill) -> i32 {
        self.effect_value(skill.as_ref())
    }

    fn inventory_space(&self) -> i32 {
        self.effect_value(Self::INVENTORY_SPACE)
    }

    fn effect_value(&self, effect: &str) -> i32 {
        self.effects()
            .iter()
            .find_map(|e| (e.code == effect).then_some(e.value))
            .unwrap_or(0)
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema>;
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
