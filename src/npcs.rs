use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::NpcSchema;
use itertools::Itertools;

use crate::{PersistedData, npcs_items::NpcsItems};

#[derive(Default, Debug)]
pub struct Npcs {
    data: RwLock<HashMap<String, Arc<NpcSchema>>>,
    api: Arc<ArtifactApi>,
    pub items: Arc<NpcsItems>,
}

impl PersistedData<HashMap<String, Arc<NpcSchema>>> for Npcs {
    const PATH: &'static str = ".cache/items.json";

    fn data_from_api(&self) -> HashMap<String, Arc<NpcSchema>> {
        self.api
            .npcs
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

impl Npcs {
    pub(crate) fn new(api: Arc<ArtifactApi>, items: Arc<NpcsItems>) -> Self {
        let npcs = Self {
            data: Default::default(),
            api,
            items,
        };
        *npcs.data.write().unwrap() = npcs.retrieve_data();
        npcs
    }

    pub fn all(&self) -> Vec<Arc<NpcSchema>> {
        self.data.read().unwrap().values().cloned().collect_vec()
    }

    pub fn get(&self, code: &str) -> Option<Arc<NpcSchema>> {
        self.data.read().unwrap().get(code).cloned()
    }

    pub fn selling(&self, code: &str) -> Vec<Arc<NpcSchema>> {
        self.items
            .all()
            .iter()
            .filter(|i| i.code == code && i.buy_price.is_some())
            .flat_map(|c| self.get(&c.npc))
            .collect_vec()
    }
}
