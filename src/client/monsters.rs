use crate::{
    CanProvideXp, CollectionClient, DataItem, DropsItems, Level, PersistData,
    client::events::EventsClient,
    simulator::{DamageType, HasEffects},
};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
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
        *monsters.data.write().unwrap() = monsters.retrieve_data();
        monsters
    }

    pub fn dropping(&self, item: &str) -> Vec<Arc<MonsterSchema>> {
        self.all()
            .into_iter()
            .filter(|m| m.drops.iter().any(|d| d.code == item))
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

impl PersistData<HashMap<String, Arc<MonsterSchema>>> for MonstersClient {
    const PATH: &'static str = ".cache/monsters.json";

    fn data_from_api(&self) -> HashMap<String, Arc<MonsterSchema>> {
        self.api
            .monsters
            .all()
            .unwrap()
            .into_iter()
            .map(|m| (m.code.clone(), Arc::new(m)))
            .collect()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl DataItem for MonstersClient {
    type Item = Arc<MonsterSchema>;
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

    fn attack_damage(&self, r#type: DamageType) -> i32 {
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

    fn resistance(&self, r#type: DamageType) -> i32 {
        match r#type {
            DamageType::Fire => self.res_fire,
            DamageType::Earth => self.res_earth,
            DamageType::Water => self.res_water,
            DamageType::Air => self.res_air,
        }
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema> {
        self.effects.iter().flatten().collect_vec()
    }
}

pub trait MonsterSchemaExt {}

impl MonsterSchemaExt for MonsterSchema {}

impl CanProvideXp for MonsterSchema {}
