use crate::{Server, gear::Slot};
use artifactsmmo_openapi::models::{CharacterSchema, ConditionOperator, ItemSchema, TaskType};
use chrono::{DateTime, Utc};
use std::{
    cmp::Ordering,
    sync::{Arc, RwLock},
    time::Duration,
};
use strum::IntoEnumIterator;

pub use character::Character;
pub use inventory::Inventory;
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
    fn server(&self) -> Arc<Server>;
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
        if self.task_progress() < self.task_total() {
            self.task_total() - self.task_progress()
        } else {
            0
        }
    }

    fn task_finished(&self) -> bool {
        !self.task().is_empty() && self.task_progress() >= self.task_total()
    }

    /// Returns the cooldown expiration timestamp of the `Character`.
    fn cooldown_expiration(&self) -> Option<DateTime<Utc>> {
        self.data()
            .cooldown_expiration
            .as_ref()
            .map(|cd| DateTime::parse_from_rfc3339(cd).ok().map(|dt| dt.to_utc()))?
    }

    fn remaining_cooldown(&self) -> Duration {
        if let Some(exp) = self.cooldown_expiration() {
            let synced = Utc::now() - *self.server().server_offset.read().unwrap();
            if synced.cmp(&exp.to_utc()) == Ordering::Less {
                return (exp.to_utc() - synced).to_std().unwrap();
            }
        }
        Duration::from_secs(0)
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
            .filter_map(|s| {
                if self.equiped_in(s) == item {
                    Some(self.quantity_in_slot(s))
                } else {
                    None
                }
            })
            .sum()
    }

    fn meets_conditions_for(&self, item: &ItemSchema) -> bool {
        item.conditions.iter().flatten().all(|c| {
            let value = if c.code == "alchemy_level" {
                self.skill_level(Skill::Alchemy)
            } else if c.code == "mining_level" {
                self.skill_level(Skill::Mining)
            } else if c.code == "woodcutting_level" {
                self.skill_level(Skill::Woodcutting)
            } else if c.code == "fishing_level" {
                self.skill_level(Skill::Fishing)
            } else {
                self.level()
            };
            match c.operator {
                ConditionOperator::Eq => (value as i32) == c.value,
                ConditionOperator::Ne => (value as i32) != c.value,
                ConditionOperator::Gt => (value as i32) > c.value,
                ConditionOperator::Lt => (value as i32) < c.value,
            }
        })
    }
}
