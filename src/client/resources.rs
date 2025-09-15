use crate::{
    CanProvideXp, CollectionClient, Data, DataItem, DropsItems, Level, PersistData,
    client::events::EventsClient,
};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::{DropRateSchema, ResourceSchema};
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

#[derive(Default, Debug)]
pub struct ResourcesClient {
    data: RwLock<HashMap<String, Arc<ResourceSchema>>>,
    api: Arc<ArtifactApi>,
    events: Arc<EventsClient>,
}

impl PersistData<HashMap<String, Arc<ResourceSchema>>> for ResourcesClient {
    const PATH: &'static str = ".cache/resources.json";

    fn data_from_api(&self) -> HashMap<String, Arc<ResourceSchema>> {
        self.api
            .resources
            .all()
            .unwrap()
            .into_iter()
            .map(|r| (r.code.clone(), Arc::new(r)))
            .collect()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl DataItem for ResourcesClient {
    type Item = Arc<ResourceSchema>;
}

impl Data for ResourcesClient {
    fn data(&self) -> RwLockReadGuard<'_, HashMap<String, Arc<ResourceSchema>>> {
        self.data.read().unwrap()
    }
}

impl CollectionClient for ResourcesClient {}

impl ResourcesClient {
    pub(crate) fn new(api: Arc<ArtifactApi>, events: Arc<EventsClient>) -> Self {
        let resources = Self {
            data: Default::default(),
            api,
            events,
        };
        *resources.data.write().unwrap() = resources.retrieve_data();
        resources
    }

    pub fn dropping(&self, item: &str) -> Vec<Arc<ResourceSchema>> {
        self.all()
            .into_iter()
            .filter(|m| m.drops.iter().any(|d| d.code == item))
            .collect_vec()
    }

    pub fn is_event(&self, code: &str) -> bool {
        self.events.all().iter().any(|e| e.content.code == code)
    }
}

impl DropsItems for ResourceSchema {
    fn drops(&self) -> &Vec<DropRateSchema> {
        &self.drops
    }
}

impl Level for ResourceSchema {
    fn level(&self) -> u32 {
        self.level as u32
    }
}

impl CanProvideXp for ResourceSchema {}
