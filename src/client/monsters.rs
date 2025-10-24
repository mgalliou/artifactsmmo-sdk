use crate::{
    CanProvideXp, Code, CollectionClient, DataEntity, DropsItems, Level, Persist,
    client::events::EventsClient,
    simulator::{HasEffects, damage_type::DamageType},
};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{
    DropRateSchema, MonsterSchema, MonsterType, SimpleEffectSchema,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct MonstersClient {
    data: RwLock<HashMap<String, Monster>>,
    api: Arc<ArtifactApi>,
    events: Arc<EventsClient>,
}

impl MonstersClient {
    pub(crate) fn new(api: Arc<ArtifactApi>, events: Arc<EventsClient>) -> Self {
        let monsters = Self {
            data: Default::default(),
            api,
            events,
        };
        *monsters.data.write().unwrap() = monsters.load();
        monsters
    }

    pub fn dropping(&self, item_code: &str) -> Vec<Monster> {
        self.all()
            .into_iter()
            .filter(|m| m.drops().iter().any(|d| d.code == item_code))
            .collect_vec()
    }

    pub fn lowest_providing_xp_at(&self, level: u32) -> Option<Monster> {
        self.all()
            .into_iter()
            .filter(|m| m.provides_xp_at(level))
            .min_by_key(|m| m.level())
    }

    pub fn highest_providing_exp(&self, level: u32) -> Option<Monster> {
        self.all()
            .into_iter()
            .filter(|m| m.provides_xp_at(level))
            .max_by_key(|m| m.level())
    }

    pub fn is_event(&self, code: &str) -> bool {
        self.events.all().iter().any(|e| e.content.code == code)
    }
}

impl Persist<HashMap<String, Monster>> for MonstersClient {
    const PATH: &'static str = ".cache/monsters.json";

    fn load_from_api(&self) -> HashMap<String, Monster> {
        self.api
            .monsters
            .get_all()
            .unwrap()
            .into_iter()
            .map(|m| (m.code.clone(), Monster(Arc::new(m))))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataEntity for MonstersClient {
    type Entity = Monster;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Monster(Arc<MonsterSchema>);

impl Monster {
    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn is_boss(&self) -> bool {
        self.0.r#type == MonsterType::Boss
    }
}

impl DropsItems for Monster {
    fn drops(&self) -> &Vec<DropRateSchema> {
        &self.0.drops
    }
}

impl Level for Monster {
    fn level(&self) -> u32 {
        self.0.level as u32
    }
}

impl Code for Monster {
    fn code(&self) -> &str {
        &self.0.code
    }
}

impl HasEffects for Monster {
    fn health(&self) -> i32 {
        self.0.hp
    }

    fn attack_dmg(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.0.attack_fire,
            DamageType::Earth => self.0.attack_earth,
            DamageType::Water => self.0.attack_water,
            DamageType::Air => self.0.attack_air,
        }
    }

    fn critical_strike(&self) -> i32 {
        self.0.critical_strike
    }

    fn res(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.0.res_fire,
            DamageType::Earth => self.0.res_earth,
            DamageType::Water => self.0.res_water,
            DamageType::Air => self.0.res_air,
        }
    }

    fn initiative(&self) -> i32 {
        self.0.initiative
    }

    fn effects(&self) -> Vec<SimpleEffectSchema> {
        self.0.effects.iter().flatten().cloned().collect_vec()
    }
}

pub trait MonsterSchemaExt {}

impl MonsterSchemaExt for Monster {}

impl CanProvideXp for Monster {}
