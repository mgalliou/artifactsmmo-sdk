use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::{Collection, Data, DataItem, PersistedData, npcs_items::NpcsItems};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::NpcSchema;
use itertools::Itertools;

#[derive(Default, Debug)]
pub struct Npcs {
    data: RwLock<HashMap<String, Arc<NpcSchema>>>,
    api: Arc<ArtifactApi>,
    pub items: Arc<NpcsItems>,
}

impl PersistedData<HashMap<String, Arc<NpcSchema>>> for Npcs {
    const PATH: &'static str = ".cache/npcs.json";

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

impl DataItem for Npcs {
    type Item = Arc<NpcSchema>;
}

impl Data for Npcs {
    fn data(&self) -> RwLockReadGuard<'_, HashMap<String, Arc<NpcSchema>>> {
        self.data.read().unwrap()
    }
}

impl Collection for Npcs {}

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

    pub fn selling(&self, code: &str) -> Vec<Arc<NpcSchema>> {
        self.items
            .all()
            .iter()
            .filter(|i| i.code == code && i.buy_price.is_some())
            .flat_map(|i| self.get(&i.npc))
            .collect_vec()
    }
}
