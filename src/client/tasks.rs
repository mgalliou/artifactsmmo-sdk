use crate::{DataItem, PersistData, TasksRewardsClient};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedRequest};
use artifactsmmo_openapi::models::{RewardsSchema, TaskFullSchema};
use sdk_derive::CollectionClient;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct TasksClient {
    data: RwLock<HashMap<String, Arc<TaskFullSchema>>>,
    pub reward: Arc<TasksRewardsClient>,
    api: Arc<ArtifactApi>,
}

impl TasksClient {
    pub(crate) fn new(api: Arc<ArtifactApi>, reward: Arc<TasksRewardsClient>) -> Self {
        let tasks = Self {
            data: Default::default(),
            reward,
            api,
        };
        *tasks.data.write().unwrap() = tasks.retrieve_data();
        tasks
    }
}

impl PersistData<HashMap<String, Arc<TaskFullSchema>>> for TasksClient {
    const PATH: &'static str = ".cache/tasks.json";

    fn data_from_api(&self) -> HashMap<String, Arc<TaskFullSchema>> {
        self.api
            .tasks
            .all()
            .unwrap()
            .into_iter()
            .map(|task| (task.code.clone(), Arc::new(task)))
            .collect()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl DataItem for TasksClient {
    type Item = Arc<TaskFullSchema>;
}

pub trait TaskFullSchemaExt {
    fn rewards_quantity(&self) -> u32 {
        self.rewards().items.iter().map(|i| i.quantity).sum()
    }

    fn rewards_slots(&self) -> u32 {
        self.rewards().items.len() as u32
    }

    fn rewards(&self) -> &RewardsSchema;
}

impl TaskFullSchemaExt for TaskFullSchema {
    fn rewards(&self) -> &RewardsSchema {
        self.rewards.as_ref()
    }
}
