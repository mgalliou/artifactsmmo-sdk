use crate::{
    CanProvideXp, Collection, Data, HasDropTable, HasLevel, PersistedData, Simulator,
    events::Events, items::DamageType, simulator::HasEffects,
};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::{DropRateSchema, ItemSchema, MonsterSchema, SimpleEffectSchema};
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};
use strum::IntoEnumIterator;

#[derive(Default, Debug)]
pub struct Monsters {
    data: RwLock<HashMap<String, Arc<MonsterSchema>>>,
    api: Arc<ArtifactApi>,
    events: Arc<Events>,
}

impl PersistedData<HashMap<String, Arc<MonsterSchema>>> for Monsters {
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

impl Data for Monsters {
    type Item = Arc<MonsterSchema>;

    fn data(&self) -> RwLockReadGuard<'_, HashMap<String, Arc<MonsterSchema>>> {
        self.data.read().unwrap()
    }
}

impl Collection for Monsters {}

impl Monsters {
    pub(crate) fn new(api: Arc<ArtifactApi>, events: Arc<Events>) -> Self {
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

impl HasDropTable for MonsterSchema {
    fn drops(&self) -> &Vec<DropRateSchema> {
        &self.drops
    }
}

impl HasLevel for MonsterSchema {
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

impl CanProvideXp for MonsterSchema {}

pub trait MonsterSchemaExt {
    fn average_damage(&self) -> f32;
    fn average_damage_against(&self, item: &ItemSchema) -> f32;
}

impl MonsterSchemaExt for MonsterSchema {
    fn average_damage(&self) -> f32 {
        DamageType::iter()
            .map(|t| Simulator::average_dmg(self.attack_damage(t), 0, self.critical_strike(), 0))
            .sum()
    }

    fn average_damage_against(&self, item: &ItemSchema) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    self.attack_damage(t),
                    0,
                    self.critical_strike(),
                    item.resistance(t),
                )
            })
            .sum::<f32>()
    }
}
