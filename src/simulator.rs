use crate::{char::Skill, gear::Gear, items::DamageType};
use artifactsmmo_openapi::models::{FightResult, MonsterSchema, SimpleEffectSchema};
use std::cmp::max;

const BASE_HP: i32 = 115;
const MAX_TURN: i32 = 100;
const HP_PER_LEVEL: i32 = 5;
const CRIT_MULTIPLIER: f32 = 1.5;

pub struct Simulator {}

impl Simulator {
    pub fn average_fight(
        level: i32,
        missing_hp: i32,
        gear: &Gear,
        monster: &MonsterSchema,
        ignore_death: bool,
    ) -> Fight {
        let base_hp = BASE_HP + HP_PER_LEVEL * level;
        let starting_hp = base_hp + gear.health() - missing_hp;
        let mut hp = starting_hp;
        let mut monster_hp = monster.hp;
        let mut turns = 1;

        loop {
            if turns % 2 == 1 {
                let damage = gear.average_damage_against(monster);
                monster_hp -= damage;
                hp += damage * gear.lifesteal() / 100;
                if monster_hp <= 0 {
                    break;
                }
                if turns > 1 {
                    monster_hp -= gear.poison()
                }
            } else {
                if hp < (base_hp + gear.health()) / 2 {
                    hp += gear.utility1.as_ref().map(|u| u.restore()).unwrap_or(0);
                    hp += gear.utility2.as_ref().map(|u| u.restore()).unwrap_or(0);
                }
                let damage = gear.avarage_damage_from(monster);
                hp -= damage;
                monster_hp += damage * gear.lifesteal() / 100;
                if turns > 2 {
                    hp -= monster.poison()
                }
                if hp <= 0 && !ignore_death {
                    break;
                }
            }
            if turns >= MAX_TURN {
                break;
            }
            turns += 1;
        }
        Fight {
            turns,
            hp,
            monster_hp,
            hp_lost: starting_hp - hp,
            result: if hp <= 0 || turns > MAX_TURN {
                FightResult::Loss
            } else {
                FightResult::Win
            },
            cd: Self::fight_cd(gear.haste(), turns),
        }
    }

    pub fn random_fight(
        level: i32,
        missing_hp: i32,
        gear: &Gear,
        monster: &MonsterSchema,
        ignore_death: bool,
    ) -> Fight {
        let base_hp = BASE_HP + HP_PER_LEVEL * level;
        let starting_hp = base_hp + gear.health() - missing_hp;
        let mut hp = starting_hp;
        let mut monster_hp = monster.hp;
        let mut turns = 1;

        loop {
            if turns % 2 == 1 {
                let hits = gear.simulate_hits_against(monster);
                for h in hits.iter() {
                    monster_hp -= h.damage;
                    if monster_hp <= 0 {
                        break;
                    }
                    if h.is_crit {
                        hp += h.damage * gear.lifesteal() / 100;
                    }
                }
                if turns > 1 {
                    monster_hp -= gear.poison()
                }
                if monster_hp <= 0 {
                    break;
                }
            } else {
                if hp < (base_hp + gear.health()) / 2 {
                    hp += gear.utility1.as_ref().map(|u| u.restore()).unwrap_or(0);
                    hp += gear.utility2.as_ref().map(|u| u.restore()).unwrap_or(0);
                }
                let hits = gear.simulate_hits_from(monster);
                for h in hits.iter() {
                    hp -= h.damage;
                    if hp <= 0 {
                        break;
                    }
                    if h.is_crit {
                        monster_hp += h.damage * monster.lifesteal() / 100;
                    }
                }
                if turns > 2 {
                    hp -= monster.poison()
                }
                if hp <= 0 && !ignore_death {
                    break;
                }
            }
            if turns >= MAX_TURN {
                break;
            }
            turns += 1;
        }
        Fight {
            turns,
            hp,
            monster_hp,
            hp_lost: starting_hp - hp,
            result: if hp <= 0 || turns > MAX_TURN {
                FightResult::Loss
            } else {
                FightResult::Win
            },
            cd: Self::fight_cd(gear.haste(), turns),
        }
    }

    /// Compute the average damage an attack will do against the given `target_resistance`.
    pub fn average_dmg(
        attack_damage: i32,
        damage_increase: i32,
        critical_strike: i32,
        target_resistance: i32,
    ) -> f32 {
        let mut dmg = attack_damage as f32 + (attack_damage as f32 * damage_increase as f32 * 0.01);
        dmg += dmg * (critical_strike as f32 / 100.0) / 2.0;
        dmg -= dmg * target_resistance as f32 * 0.01;
        dmg
    }

    pub fn critless_dmg(attack_damage: i32, damage_increase: i32, target_resistance: i32) -> f32 {
        let mut dmg = attack_damage as f32 + (attack_damage as f32 * damage_increase as f32 * 0.01);
        dmg -= dmg * target_resistance as f32 * 0.01;
        dmg
    }

    pub fn simulate_dmg(
        attack_damage: i32,
        damage_increase: i32,
        critical_strike: i32,
        target_resistance: i32,
    ) -> f32 {
        let mut dmg = attack_damage as f32 + (attack_damage as f32 * damage_increase as f32 * 0.01);
        if rand::random_range(0..=100) <= critical_strike {
            dmg *= CRIT_MULTIPLIER
        }
        dmg -= dmg * target_resistance as f32 * 0.01;
        dmg
    }

    pub fn simulate_hit(
        attack_damage: i32,
        damage_increase: i32,
        critical_strike: i32,
        r#type: DamageType,
        target_resistance: i32,
    ) -> Hit {
        let mut is_crit = false;
        let mut damage =
            attack_damage as f32 + (attack_damage as f32 * damage_increase as f32 * 0.01);
        if rand::random_range(0..=100) <= critical_strike {
            damage *= 1.5;
            is_crit = true
        }
        damage -= damage * target_resistance as f32 * 0.01;
        Hit {
            r#type,
            damage: damage.round() as i32,
            is_crit,
        }
    }

    pub fn time_to_rest(health: i32) -> i32 {
        health / 5 + if health % 5 > 0 { 1 } else { 0 }
    }

    fn fight_cd(haste: i32, turns: i32) -> i32 {
        max(
            5,
            ((turns * 2) as f32 - (haste as f32 * 0.01) * (turns * 2) as f32).round() as i32,
        )
    }

    pub fn gather_cd(resource_level: i32, cooldown_reduction: i32) -> i32 {
        ((30.0 + (resource_level as f32 / 2.0)) * (1.0 + cooldown_reduction as f32 / 100.0)) as i32
    }
}

#[derive(Debug)]
pub struct Fight {
    pub turns: i32,
    pub hp: i32,
    pub monster_hp: i32,
    pub hp_lost: i32,
    pub result: FightResult,
    pub cd: i32,
}

pub struct Hit {
    pub r#type: DamageType,
    pub damage: i32,
    pub is_crit: bool,
}

pub trait HasEffects {
    fn health(&self) -> i32 {
        let hp = self.effect_value("hp");
        if hp < 1 {
            self.effect_value("boost_hp")
        } else {
            hp
        }
    }

    fn heal(&self) -> i32 {
        self.effect_value("heal")
    }

    fn restore(&self) -> i32 {
        self.effect_value("restore")
    }

    fn haste(&self) -> i32 {
        self.effect_value("haste")
    }

    fn attack_damage(&self, r#type: DamageType) -> i32 {
        self.effect_value(&("attack_".to_string() + r#type.as_ref()))
    }

    fn damage_increase(&self, r#type: DamageType) -> i32 {
        self.effect_value(&("dmg_".to_string() + r#type.as_ref()))
            + self.effect_value(&("boost_dmg_".to_string() + r#type.as_ref()))
            + self.effect_value("dmg")
    }

    fn critical_strike(&self) -> i32 {
        self.effect_value("critical_strike")
    }

    fn poison(&self) -> i32 {
        self.effect_value("poison")
    }

    fn lifesteal(&self) -> i32 {
        self.effect_value("lifesteal")
    }

    fn resistance(&self, r#type: DamageType) -> i32 {
        self.effect_value(&("res_".to_string() + r#type.as_ref()))
    }

    fn wisdom(&self) -> i32 {
        self.effect_value("wisdom")
    }

    fn prospecting(&self) -> i32 {
        self.effect_value("prospecting")
    }

    fn skill_cooldown_reduction(&self, skill: Skill) -> i32 {
        self.effect_value(skill.as_ref())
    }

    fn inventory_space(&self) -> i32 {
        self.effect_value("inventory_space")
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
