use crate::{DataEntity, Persist};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{self};
use sdk_derive::CollectionClient;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

type NpcItem = Arc<models::NpcItem>;

#[derive(Default, Debug, CollectionClient)]
pub struct NpcsItemsClient {
    data: RwLock<HashMap<String, NpcItem>>,
    api: Arc<ArtifactApi>,
}

impl NpcsItemsClient {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let npcs_items = Self {
            data: Default::default(),
            api,
        };
        *npcs_items.data.write().unwrap() = npcs_items.load();
        npcs_items
    }
}

impl Persist<HashMap<String, NpcItem>> for NpcsItemsClient {
    const PATH: &'static str = ".cache/npcs_items.json";

    fn load_from_api(&self) -> HashMap<String, NpcItem> {
        self.api
            .npcs
            .get_items()
            .unwrap()
            .into_iter()
            .map(|npc| (npc.code.clone(), Arc::new(npc)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataEntity for NpcsItemsClient {
    type Entity = NpcItem;
}

pub trait NpcItemExt {
    fn buy_price(&self) -> Option<u32>;
}

impl NpcItemExt for NpcItem {
    fn buy_price(&self) -> Option<u32> {
        self.buy_price.map(|p| p as u32)
    }
}
