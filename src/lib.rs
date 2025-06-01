use artifactsmmo_openapi::models::{FightSchema, RewardsSchema, SkillDataSchema, SkillInfoSchema};
use fs_extra::file::{read_to_string, write_all};
use log::error;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use artifactsmmo_openapi::models;

pub use account::Account;
pub use bank::Bank;
pub use char::Character;
pub use client::Client;
pub use simulator::Simulator;
//pub use consts::{};
pub use events::Events;
pub use gear::Gear;
pub use items::Items;
pub use maps::Maps;
pub use monsters::Monsters;
pub use resources::Resources;
pub use server::Server;
pub use tasks::Tasks;
pub use tasks_rewards::TasksRewards;

pub mod account;
pub mod bank;
pub mod char;
pub mod client;
pub mod consts;
pub mod events;
pub mod gear;
pub mod item_code;
pub mod items;
pub mod maps;
pub mod monsters;
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
