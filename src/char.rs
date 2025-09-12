use crate::{gear::Slot, items::ItemCondition};
use artifactsmmo_openapi::models::{CharacterSchema, ConditionOperator, ItemSchema, TaskType};
use chrono::{DateTime, Utc, format::Item};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};
use strum::IntoEnumIterator;

pub use character::Character;
pub use inventory::InventoryClient;
pub use skill::Skill;

pub mod action;
pub mod character;
pub mod error;
pub mod inventory;
pub mod request_handler;
pub mod skill;

pub type CharacterData = Arc<RwLock<Arc<CharacterSchema>>>;

pub trait HasCharacterData {
    fn data(&self) -> Arc<CharacterSchema>;
    fn refresh_data(&self);
    fn update_data(&self, schema: CharacterSchema);

    fn name(&self) -> String {
        self.data().name.to_owned()
    }

    /// Returns the `Character` position (coordinates).
    fn position(&self) -> (i32, i32) {
        let d = self.data();
        (d.x, d.y)
    }

    fn level(&self) -> u32 {
        self.data().level as u32
    }

    /// Returns the `Character` level in the given `skill`.
    fn skill_level(&self, skill: Skill) -> u32 {
        let d = self.data();
        (match skill {
            Skill::Combat => d.level,
            Skill::Mining => d.mining_level,
            Skill::Woodcutting => d.woodcutting_level,
            Skill::Fishing => d.fishing_level,
            Skill::Weaponcrafting => d.weaponcrafting_level,
            Skill::Gearcrafting => d.gearcrafting_level,
            Skill::Jewelrycrafting => d.jewelrycrafting_level,
            Skill::Cooking => d.cooking_level,
            Skill::Alchemy => d.alchemy_level,
        }) as u32
    }

    fn skill_xp(&self, skill: Skill) -> i32 {
        let d = self.data();
        match skill {
            Skill::Combat => d.xp,
            Skill::Mining => d.mining_xp,
            Skill::Woodcutting => d.woodcutting_xp,
            Skill::Fishing => d.fishing_xp,
            Skill::Weaponcrafting => d.weaponcrafting_xp,
            Skill::Gearcrafting => d.gearcrafting_xp,
            Skill::Jewelrycrafting => d.jewelrycrafting_xp,
            Skill::Cooking => d.cooking_xp,
            Skill::Alchemy => d.alchemy_xp,
        }
    }

    fn skill_max_xp(&self, skill: Skill) -> i32 {
        let d = self.data();
        match skill {
            Skill::Combat => d.max_xp,
            Skill::Mining => d.mining_max_xp,
            Skill::Woodcutting => d.woodcutting_max_xp,
            Skill::Fishing => d.fishing_max_xp,
            Skill::Weaponcrafting => d.weaponcrafting_max_xp,
            Skill::Gearcrafting => d.gearcrafting_max_xp,
            Skill::Jewelrycrafting => d.jewelrycrafting_max_xp,
            Skill::Cooking => d.cooking_max_xp,
            Skill::Alchemy => d.alchemy_max_xp,
        }
    }

    fn max_health(&self) -> i32 {
        self.data().max_hp
    }

    fn health(&self) -> i32 {
        self.data().hp
    }

    fn missing_hp(&self) -> i32 {
        self.max_health() - self.health()
    }

    fn gold(&self) -> u32 {
        self.data().gold as u32
    }

    fn quantity_in_slot(&self, s: Slot) -> u32 {
        match s {
            Slot::Utility1 => self.data().utility1_slot_quantity,
            Slot::Utility2 => self.data().utility2_slot_quantity,
            Slot::Weapon
            | Slot::Shield
            | Slot::Helmet
            | Slot::BodyArmor
            | Slot::LegArmor
            | Slot::Boots
            | Slot::Ring1
            | Slot::Ring2
            | Slot::Amulet
            | Slot::Artifact1
            | Slot::Artifact2
            | Slot::Artifact3
            | Slot::Bag
            | Slot::Rune => {
                if self.equiped_in(s).is_empty() {
                    0
                } else {
                    1
                }
            }
        }
    }

    fn task(&self) -> String {
        self.data().task.to_owned()
    }

    fn task_type(&self) -> Option<TaskType> {
        match self.data().task_type.as_str() {
            "monsters" => Some(TaskType::Monsters),
            "items" => Some(TaskType::Items),
            _ => None,
        }
    }

    fn task_progress(&self) -> u32 {
        self.data().task_progress as u32
    }

    fn task_total(&self) -> u32 {
        self.data().task_total as u32
    }

    fn task_missing(&self) -> u32 {
        self.task_total().saturating_sub(self.task_progress())
    }

    fn task_finished(&self) -> bool {
        !self.task().is_empty() && self.task_missing() < 1
    }

    /// Returns the cooldown expiration timestamp of the `Character`.
    fn cooldown_expiration(&self) -> Option<DateTime<Utc>> {
        self.data()
            .cooldown_expiration
            .as_ref()
            .map(|cd| DateTime::parse_from_rfc3339(cd).ok().map(|dt| dt.to_utc()))?
    }

    fn equiped_in(&self, slot: Slot) -> String {
        let d = self.data();
        match slot {
            Slot::Weapon => &d.weapon_slot,
            Slot::Shield => &d.shield_slot,
            Slot::Helmet => &d.helmet_slot,
            Slot::BodyArmor => &d.body_armor_slot,
            Slot::LegArmor => &d.leg_armor_slot,
            Slot::Boots => &d.boots_slot,
            Slot::Ring1 => &d.ring1_slot,
            Slot::Ring2 => &d.ring2_slot,
            Slot::Amulet => &d.amulet_slot,
            Slot::Artifact1 => &d.artifact1_slot,
            Slot::Artifact2 => &d.artifact2_slot,
            Slot::Artifact3 => &d.artifact3_slot,
            Slot::Utility1 => &d.utility1_slot,
            Slot::Utility2 => &d.utility2_slot,
            Slot::Bag => &d.bag_slot,
            Slot::Rune => &d.rune_slot,
        }
        .clone()
    }

    fn has_equiped(&self, item: &str) -> u32 {
        Slot::iter()
            .filter_map(|s| (self.equiped_in(s) == item).then_some(self.quantity_in_slot(s)))
            .sum()
    }

    fn meets_conditions_for(&self, item: &ItemSchema) -> bool {
        item.conditions.iter().flatten().all(|c| {
            let Ok(condition) = ItemCondition::from_str(&c.code) else {
                return false;
            };
            let value = self.skill_level(condition.into());
            match c.operator {
                ConditionOperator::Eq => (value as i32) == c.value,
                ConditionOperator::Ne => (value as i32) != c.value,
                ConditionOperator::Gt => (value as i32) > c.value,
                ConditionOperator::Lt => (value as i32) < c.value,
            }
        })
    }
}
