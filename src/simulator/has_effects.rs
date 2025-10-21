use artifactsmmo_openapi::models::SimpleEffectSchema;
use crate::{Simulator, Skill, damage_type::DamageType, hit::Hit};

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

    fn attack_dmg(&self, r#type: DamageType) -> i32 {
        self.effect_value(r#type.into_attack())
    }

    fn dmg_increase(&self, r#type: DamageType) -> i32 {
        self.effect_value(DMG)
            + self.effect_value(r#type.into_dmg())
            + self.effect_value(r#type.into_boost_dmg())
    }

    fn res(&self, r#type: DamageType) -> i32 {
        self.effect_value(r#type.into_res())
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

    fn hits_against(&self, target: &dyn HasEffects, averaged: bool) -> Vec<Hit> {
        let mut is_crit = false;
        if !averaged {
            is_crit = rand::random_range(0..=100) <= self.critical_strike();
        }
        DamageType::iter()
            .filter_map(|t| {
                let attack_dmg = self.attack_dmg(t);
                (attack_dmg > 0).then_some(if averaged {
                    Hit::averaged(
                        self.attack_dmg(t),
                        self.dmg_increase(t),
                        self.critical_strike(),
                        t,
                        target.res(t),
                    )
                } else {
                    Hit::new(
                        self.attack_dmg(t),
                        self.dmg_increase(t),
                        t,
                        target.res(t),
                        is_crit,
                    )
                })
            })
            .collect_vec()
    }

    fn critless_dmg_against(&self, target: &dyn HasEffects) -> i32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(self.attack_dmg(t), self.dmg_increase(t), 0, target.res(t))
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
        target.average_dmg() - target.average_dmg_against(self)
    }

    fn average_dmg_against(&self, target: &dyn HasEffects) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(self.attack_dmg(t), 0, self.critical_strike(), target.res(t))
            })
            .sum()
    }

    // Returns the average attack damage done by the `self` entity against the `target ` entity with additionnnal effects from the `boost` entity
    // damage `boost`
    fn average_dmg_against_with(&self, boost: &dyn HasEffects, target: &dyn HasEffects) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    self.attack_dmg(t),
                    boost.dmg_increase(t),
                    self.critical_strike() + boost.critical_strike(),
                    target.res(t),
                )
            })
            .sum()
    }

    fn average_dmg(&self) -> f32 {
        DamageType::iter()
            .map(|t| Simulator::average_dmg(self.attack_dmg(t), 0, self.critical_strike(), 0))
            .sum()
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema>;
}
