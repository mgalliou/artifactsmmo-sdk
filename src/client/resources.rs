use crate::{
    CanProvideXp, Code, CollectionClient, DataEntity, DropsItems, Level, Persist,
    client::events::EventsClient, skill::Skill,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{DropRateSchema, ResourceSchema};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct ResourcesClient {
    data: RwLock<HashMap<String, Resource>>,
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

    pub fn dropping(&self, item_code: &str) -> Vec<Resource> {
        self.all()
            .into_iter()
            .filter(|r| r.drops().iter().any(|d| d.code == item_code))
            .collect_vec()
    }

    pub fn is_event(&self, code: &str) -> bool {
        self.events.all().iter().any(|e| e.content.code == code)
    }
}

impl Persist<HashMap<String, Resource>> for ResourcesClient {
    const PATH: &'static str = ".cache/resources.json";

    fn load_from_api(&self) -> HashMap<String, Resource> {
        self.api
            .resources
            .get_all()
            .unwrap()
            .into_iter()
            .map(|r| (r.code.clone(), Resource(Arc::new(r))))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataEntity for ResourcesClient {
    type Entity = Resource;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(Arc<ResourceSchema>);

impl Resource {
    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn skill(&self) -> Skill {
        self.0.skill.into()
    }
}

impl DropsItems for Resource {
    fn drops(&self) -> &Vec<DropRateSchema> {
        &self.0.drops
    }
}

impl Code for Resource {
    fn code(&self) -> &str {
        &self.0.code
    }
}

impl Level for Resource {
    fn level(&self) -> u32 {
        self.0.level as u32
    }
}

impl CanProvideXp for Resource {}
