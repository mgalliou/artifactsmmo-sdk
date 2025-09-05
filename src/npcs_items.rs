use crate::PersistedData;
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::NpcItem;
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug)]
pub struct NpcsItems {
    data: RwLock<HashMap<String, Arc<NpcItem>>>,
    api: Arc<ArtifactApi>,
}

impl PersistedData<HashMap<String, Arc<NpcItem>>> for NpcsItems {
    const PATH: &'static str = ".cache/npcs_items.json";

    fn data_from_api(&self) -> HashMap<String, Arc<NpcItem>> {
        self.api
            .npcs_items
            .all()
            .unwrap()
            .into_iter()
            .map(|npc| (npc.code.clone(), Arc::new(npc)))
            .collect()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl NpcsItems {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let npcs_items = Self {
            data: Default::default(),
            api,
        };
        *npcs_items.data.write().unwrap() = npcs_items.retrieve_data();
        npcs_items
    }

    pub fn all(&self) -> Vec<Arc<NpcItem>> {
        self.data.read().unwrap().values().cloned().collect_vec()
    }

    pub fn get(&self, code: &str) -> Option<Arc<NpcItem>> {
        self.data.read().unwrap().get(code).cloned()
    }
}

pub trait NpcItemExt {
    fn buy_price(&self) -> Option<u32>;
}

impl NpcItemExt for NpcItem {
    fn buy_price(&self) -> Option<u32> {
        self.buy_price.map(|p| p as u32)
    }
}
