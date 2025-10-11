use crate::{CollectionClient, DataEntity, Persist, client::npcs_items::NpcsItemsClient};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::NpcSchema;
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct NpcsClient {
    data: RwLock<HashMap<String, Arc<NpcSchema>>>,
    api: Arc<ArtifactApi>,
    pub items: Arc<NpcsItemsClient>,
}

impl NpcsClient {
    pub(crate) fn new(api: Arc<ArtifactApi>, items: Arc<NpcsItemsClient>) -> Self {
        let npcs = Self {
            data: Default::default(),
            api,
            items,
        };
        *npcs.data.write().unwrap() = npcs.load();
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

impl Persist<HashMap<String, Arc<NpcSchema>>> for NpcsClient {
    const PATH: &'static str = ".cache/npcs.json";

    fn load_from_api(&self) -> HashMap<String, Arc<NpcSchema>> {
        self.api
            .npcs
            .get_all()
            .unwrap()
            .into_iter()
            .map(|npc| (npc.code.clone(), Arc::new(npc)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataEntity for NpcsClient {
    type Entity = Arc<NpcSchema>;
}
