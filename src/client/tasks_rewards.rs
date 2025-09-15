use crate::{DataItem, PersistData};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::DropRateSchema;
use sdk_derive::CollectionClient;
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
        *rewards.data.write().unwrap() = rewards.retrieve_data();
        rewards
    }
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
