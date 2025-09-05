use artifactsmmo_openapi::models::{
    DropRateSchema, DropSchema, FightSchema, RewardsSchema, SimpleItemSchema, SkillDataSchema,
    SkillInfoSchema,
};
use fs_extra::file::{read_to_string, write_all};
use log::error;
use serde::{Deserialize, Serialize};
use std::path::Path;
use strum_macros::{AsRefStr, Display, EnumIs, EnumIter, EnumString};

pub use artifactsmmo_openapi::models;

pub use account::Account;
pub use bank::Bank;
pub use char::Character;
pub use client::Client;
pub use consts::*;
pub use events::Events;
pub use gear::Gear;
pub use items::Items;
pub use maps::Maps;
pub use monsters::Monsters;
pub use resources::Resources;
pub use server::Server;
pub use simulator::Simulator;
pub use tasks::Tasks;
pub use tasks_rewards::TasksRewards;

pub mod account;
pub mod bank;
pub mod char;
pub mod client;
pub mod consts;
pub mod error;
pub mod events;
pub mod gear;
pub mod item_code;
pub mod items;
pub mod maps;
pub mod monsters;
pub mod npcs;
pub mod npcs_items;
pub mod resources;
pub mod server;
pub mod simulator;
pub mod tasks;
pub mod tasks_rewards;

pub trait PersistedData<D: for<'a> Deserialize<'a> + Serialize> {
    const PATH: &'static str;

    fn retrieve_data(&self) -> D {
        if let Ok(data) = self.data_from_file::<D>() {
            data
        } else {
            let data = self.data_from_api();
            if let Err(e) = Self::persist_data(&data) {
                error!("failed to persist data: {}", e);
            }
            data
        }
    }
    fn data_from_api(&self) -> D;
    fn data_from_file<T: for<'a> Deserialize<'a>>(&self) -> Result<T, Box<dyn std::error::Error>> {
        Ok(serde_json::from_str(&read_to_string(Path::new(
            Self::PATH,
        ))?)?)
    }
    fn persist_data<T: Serialize>(data: T) -> Result<(), Box<dyn std::error::Error>> {
        Ok(write_all(
            Path::new(Self::PATH),
            &serde_json::to_string_pretty(&data)?,
        )?)
    }
    fn refresh_data(&self);
}

pub trait HasDrops {
    fn amount_of(&self, item: &str) -> i32;
}

impl HasDrops for FightSchema {
    fn amount_of(&self, item: &str) -> i32 {
        self.drops
            .iter()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }
}

impl HasDrops for SkillDataSchema {
    fn amount_of(&self, item: &str) -> i32 {
        self.details
            .items
            .iter()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }
}

impl HasDrops for SkillInfoSchema {
    fn amount_of(&self, item: &str) -> i32 {
        self.items
            .iter()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }
}

impl HasDrops for RewardsSchema {
    fn amount_of(&self, item: &str) -> i32 {
        self.items
            .iter()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }
}

impl HasDrops for Vec<SimpleItemSchema> {
    fn amount_of(&self, item: &str) -> i32 {
        self.iter()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }
}

impl HasDrops for Vec<DropSchema> {
    fn amount_of(&self, item: &str) -> i32 {
        self.iter()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }
}

pub trait HasDropTable {
    fn drop_rate(&self, item: &str) -> Option<i32> {
        self.drops().iter().find(|i| i.code == item).map(|i| i.rate)
    }

    fn average_drop_quantity(&self) -> i32 {
        self.drops()
            .iter()
            .map(|i| 1.0 / i.rate as f32 * (i.max_quantity + i.min_quantity) as f32 / 2.0)
            .sum::<f32>()
            .ceil() as i32
    }

    fn average_drop_slots(&self) -> i32 {
        self.drops()
            .iter()
            .map(|i| 1.0 / i.rate as f32)
            .sum::<f32>()
            .ceil() as i32
    }

    fn max_drop_quantity(&self) -> i32 {
        self.drops().iter().map(|i| i.max_quantity).sum()
    }

    fn drops(&self) -> &Vec<DropRateSchema>;
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

pub struct DropSchemas<'a>(pub &'a Vec<DropSchema>);

impl std::fmt::Display for DropSchemas<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items: String = "".to_string();
        for item in self.0 {
            if !items.is_empty() {
                items.push_str(", ");
            }
            items.push_str(&format!("'{}'x{}", item.code, item.quantity));
        }
        write!(f, "{}", items)
    }
}

pub struct SimpleItemSchemas<'a>(pub &'a Vec<SimpleItemSchema>);

impl std::fmt::Display for SimpleItemSchemas<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items: String = "".to_string();
        for item in self.0 {
            if !items.is_empty() {
                items.push_str(", ");
            }
            items.push_str(&format!("'{}'x{}", item.code, item.quantity));
        }
        write!(f, "{}", items)
    }
}
