use crate::{
    CanProvideXp, CollectionClient, DataEntity, DropsItems, Level, Persist,
    client::events::EventsClient,
    simulator::{DamageType, HasEffects},
};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{DropRateSchema, MonsterSchema, SimpleEffectSchema};
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct MonstersClient {
    data: RwLock<HashMap<String, Arc<MonsterSchema>>>,
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

    pub fn dropping(&self, item_code: &str) -> Vec<Arc<MonsterSchema>> {
        self.all()
            .into_iter()
            .filter(|m| m.drops.iter().any(|d| d.code == item_code))
            .collect_vec()
    }

    pub fn lowest_providing_xp_at(&self, level: u32) -> Option<Arc<MonsterSchema>> {
        self.all()
            .into_iter()
            .filter(|m| m.provides_xp_at(level))
            .min_by_key(|m| m.level)
    }

    pub fn highest_providing_exp(&self, level: u32) -> Option<Arc<MonsterSchema>> {
        self.all()
            .into_iter()
            .filter(|m| m.provides_xp_at(level))
            .max_by_key(|m| m.level)
    }

    pub fn is_event(&self, code: &str) -> bool {
        self.events.all().iter().any(|e| e.content.code == code)
    }
}

impl Persist<HashMap<String, Arc<MonsterSchema>>> for MonstersClient {
    const PATH: &'static str = ".cache/monsters.json";

    fn load_from_api(&self) -> HashMap<String, Arc<MonsterSchema>> {
        self.api
            .monsters
            .get_all()
            .unwrap()
            .into_iter()
            .map(|m| (m.code.clone(), Arc::new(m)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataEntity for MonstersClient {
    type Entity = Arc<MonsterSchema>;
}

impl DropsItems for MonsterSchema {
    fn drops(&self) -> &Vec<DropRateSchema> {
        &self.drops
    }
}

impl Level for MonsterSchema {
    fn level(&self) -> u32 {
        self.level as u32
    }
}

impl HasEffects for MonsterSchema {
    fn health(&self) -> i32 {
        self.hp
    }

    fn attack_dmg(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.attack_fire,
            DamageType::Earth => self.attack_earth,
            DamageType::Water => self.attack_water,
            DamageType::Air => self.attack_air,
        }
    }

    fn critical_strike(&self) -> i32 {
        self.critical_strike
    }

    fn res(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.res_fire,
            DamageType::Earth => self.res_earth,
            DamageType::Water => self.res_water,
            DamageType::Air => self.res_air,
        }
    }

    fn initiative(&self) -> i32 {
        self.initiative
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema> {
        self.effects.iter().flatten().collect_vec()
    }
}

pub trait MonsterSchemaExt {}

impl MonsterSchemaExt for MonsterSchema {}

impl CanProvideXp for MonsterSchema {}
