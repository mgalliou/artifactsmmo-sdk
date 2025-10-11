use crate::{
    CanProvideXp, CollectionClient, DataEntity, DropsItems, Level, Persist,
    client::events::EventsClient,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{DropRateSchema, ResourceSchema};
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct ResourcesClient {
    data: RwLock<HashMap<String, Arc<ResourceSchema>>>,
    api: Arc<ArtifactApi>,
    events: Arc<EventsClient>,
}

impl ResourcesClient {
    pub(crate) fn new(api: Arc<ArtifactApi>, events: Arc<EventsClient>) -> Self {
        let resources = Self {
            data: Default::default(),
            api,
            events,
        };
        *resources.data.write().unwrap() = resources.load();
        resources
    }

    pub fn dropping(&self, item_code: &str) -> Vec<Arc<ResourceSchema>> {
        self.all()
            .into_iter()
            .filter(|m| m.drops.iter().any(|d| d.code == item_code))
            .collect_vec()
    }

    pub fn is_event(&self, code: &str) -> bool {
        self.events.all().iter().any(|e| e.content.code == code)
    }
}

impl Persist<HashMap<String, Arc<ResourceSchema>>> for ResourcesClient {
    const PATH: &'static str = ".cache/resources.json";

    fn load_from_api(&self) -> HashMap<String, Arc<ResourceSchema>> {
        self.api
            .resources
            .get_all()
            .unwrap()
            .into_iter()
            .map(|r| (r.code.clone(), Arc::new(r)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataEntity for ResourcesClient {
    type Entity = Arc<ResourceSchema>;
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
