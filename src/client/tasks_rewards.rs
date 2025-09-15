use crate::{CollectionClient, Data, DataItem, PersistData};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::DropRateSchema;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

#[derive(Default, Debug)]
pub struct TasksRewardsClient {
    data: RwLock<HashMap<String, Arc<DropRateSchema>>>,
    api: Arc<ArtifactApi>,
}

impl PersistData<HashMap<String, Arc<DropRateSchema>>> for TasksRewardsClient {
    const PATH: &'static str = ".cache/tasks_rewards.json";

    fn data_from_api(&self) -> HashMap<String, Arc<DropRateSchema>> {
        self.api
            .tasks_reward
            .all()
            .unwrap()
            .into_iter()
            .map(|tr| (tr.code.clone(), Arc::new(tr)))
            .collect()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl DataItem for TasksRewardsClient {
    type Item = Arc<DropRateSchema>;
}

impl Data for TasksRewardsClient {
    fn data(&self) -> RwLockReadGuard<'_, HashMap<String, Self::Item>> {
        self.data.read().unwrap()
    }
}

impl CollectionClient for TasksRewardsClient {}

impl TasksRewardsClient {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let rewards = Self {
            data: Default::default(),
            api,
        };
        *rewards.data.write().unwrap() = rewards.retrieve_data();
        rewards
    }
}
