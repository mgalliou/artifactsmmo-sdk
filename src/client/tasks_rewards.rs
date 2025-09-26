use crate::{CollectionClient, DataItem, Persist};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::DropRateSchema;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct TasksRewardsClient {
    data: RwLock<HashMap<String, Arc<DropRateSchema>>>,
    api: Arc<ArtifactApi>,
}

impl TasksRewardsClient {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let rewards = Self {
            data: Default::default(),
            api,
        };
        *rewards.data.write().unwrap() = rewards.load();
        rewards
    }

    pub fn max_quantity(&self) -> u32 {
        self.all()
            .iter()
            .max_by_key(|i| i.max_quantity)
            .map_or(0, |i| i.max_quantity)
    }
}

impl Persist<HashMap<String, Arc<DropRateSchema>>> for TasksRewardsClient {
    const PATH: &'static str = ".cache/tasks_rewards.json";

    fn load_from_api(&self) -> HashMap<String, Arc<DropRateSchema>> {
        self.api
            .tasks
            .get_rewards()
            .unwrap()
            .into_iter()
            .map(|tr| (tr.code.clone(), Arc::new(tr)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataItem for TasksRewardsClient {
    type Item = Arc<DropRateSchema>;
}
