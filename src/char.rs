use crate::gear::Slot;
use artifactsmmo_openapi::models::{CharacterSchema, TaskType};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

pub use character::Character;
pub use inventory::Inventory;
pub use skill::Skill;

pub mod action;
pub mod character;
pub mod inventory;
pub mod request_handler;
pub mod skill;

pub type CharacterData = Arc<RwLock<Arc<CharacterSchema>>>;

pub trait HasCharacterData {
    async fn data(&self) -> Arc<CharacterSchema>;

    async fn name(&self) -> String {
        self.data().await.name.to_owned()
    }

    /// Returns the `Character` position (coordinates).
    async fn position(&self) -> (i32, i32) {
        let d = self.data().await;
        (d.x, d.y)
    }

    async fn level(&self) -> i32 {
        self.data().await.level
    }

    async fn skill_xp(&self, skill: Skill) -> i32 {
        let d = self.data().await;
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

    async fn skill_max_xp(&self, skill: Skill) -> i32 {
        let d = self.data().await;
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

    async fn max_health(&self) -> i32 {
        self.data().await.max_hp
    }

    async fn health(&self) -> i32 {
        self.data().await.hp
    }

    async fn missing_hp(&self) -> i32 {
        self.max_health().await - self.health().await
    }

    /// Returns the `Character` level in the given `skill`.
    async fn skill_level(&self, skill: Skill) -> i32 {
        let d = self.data().await;
        match skill {
            Skill::Combat => d.level,
            Skill::Mining => d.mining_level,
            Skill::Woodcutting => d.woodcutting_level,
            Skill::Fishing => d.fishing_level,
            Skill::Weaponcrafting => d.weaponcrafting_level,
            Skill::Gearcrafting => d.gearcrafting_level,
            Skill::Jewelrycrafting => d.jewelrycrafting_level,
            Skill::Cooking => d.cooking_level,
            Skill::Alchemy => d.alchemy_level,
        }
    }

    async fn gold(&self) -> i32 {
        self.data().await.gold
    }

    // TODO: return 1 if item is equipped
    async fn quantity_in_slot(&self, s: Slot) -> i32 {
        match s {
            Slot::Utility1 => self.data().await.utility1_slot_quantity,
            Slot::Utility2 => self.data().await.utility2_slot_quantity,
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
            | Slot::Rune => 1,
        }
    }

    async fn task(&self) -> String {
        self.data().await.task.to_owned()
    }

    async fn task_type(&self) -> Option<TaskType> {
        match self.data().await.task_type.as_str() {
            "monsters" => Some(TaskType::Monsters),
            "items" => Some(TaskType::Items),
            _ => None,
        }
    }

    async fn task_progress(&self) -> i32 {
        self.data().await.task_progress
    }

    async fn task_total(&self) -> i32 {
        self.data().await.task_total
    }

    async fn task_missing(&self) -> i32 {
        self.task_total().await - self.task_progress().await
    }

    async fn task_finished(&self) -> bool {
        !self.task().await.is_empty() && self.task_progress().await >= self.task_total().await
    }

    /// Returns the cooldown expiration timestamp of the `Character`.
    async fn cooldown_expiration(&self) -> Option<DateTime<Utc>> {
        self.data().await
            .cooldown_expiration
            .as_ref()
            .map(|cd| DateTime::parse_from_rfc3339(cd).ok().map(|dt| dt.to_utc()))?
    }

    //TODO:
    // Returns the current `Gear` of the `Character`, containing item schemas.
    // fn gear(&self) -> Gear {
    //     let d = self.data();
    //     Gear {
    //         weapon: ITEMS.get(&d.weapon_slot),
    //         shield: ITEMS.get(&d.shield_slot),
    //         helmet: ITEMS.get(&d.helmet_slot),
    //         body_armor: ITEMS.get(&d.body_armor_slot),
    //         leg_armor: ITEMS.get(&d.leg_armor_slot),
    //         boots: ITEMS.get(&d.boots_slot),
    //         ring1: ITEMS.get(&d.ring1_slot),
    //         ring2: ITEMS.get(&d.ring2_slot),
    //         amulet: ITEMS.get(&d.amulet_slot),
    //         artifact1: ITEMS.get(&d.artifact1_slot),
    //         artifact2: ITEMS.get(&d.artifact2_slot),
    //         artifact3: ITEMS.get(&d.artifact3_slot),
    //         utility1: ITEMS.get(&d.utility1_slot),
    //         utility2: ITEMS.get(&d.utility2_slot),
    //     }
    // }

    /// Returns the item equiped in the `given` slot.
    async fn equiped_in(&self, slot: Slot) -> String {
        let d = self.data().await;
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
}
