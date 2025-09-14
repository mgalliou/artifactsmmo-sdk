use crate::{Collection, Data, DataItem, PersistedData};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::DropRateSchema;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

#[derive(Default, Debug)]
pub struct TasksRewards {
    data: RwLock<HashMap<String, Arc<DropRateSchema>>>,
    api: Arc<ArtifactApi>,
}

impl PersistedData<HashMap<String, Arc<DropRateSchema>>> for TasksRewards {
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

impl DataItem for TasksRewards {
    type Item = Arc<DropRateSchema>;
}

impl Data for TasksRewards {
    fn data(&self) -> RwLockReadGuard<'_, HashMap<String, Self::Item>> {
        self.data.read().unwrap()
    }
}

impl Collection for TasksRewards {}

impl TasksRewards {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let rewards = Self {
            data: Default::default(),
            api,
        };
        *rewards.data.write().unwrap() = rewards.retrieve_data();
        rewards
    }
}
