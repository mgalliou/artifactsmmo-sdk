use crate::{DataItem, PersistData};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::NpcItem;
use sdk_derive::CollectionClient;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct NpcsItemsClient {
    data: RwLock<HashMap<String, Arc<NpcItem>>>,
    api: Arc<ArtifactApi>,
}

impl NpcsItemsClient {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let npcs_items = Self {
            data: Default::default(),
            api,
        };
        *npcs_items.data.write().unwrap() = npcs_items.retrieve_data();
        npcs_items
    }
}

impl PersistData<HashMap<String, Arc<NpcItem>>> for NpcsItemsClient {
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

impl DataItem for NpcsItemsClient {
    type Item = Arc<NpcItem>;
}

pub trait NpcItemExt {
    fn buy_price(&self) -> Option<u32>;
}

impl NpcItemExt for NpcItem {
    fn buy_price(&self) -> Option<u32> {
        self.buy_price.map(|p| p as u32)
    }
}
