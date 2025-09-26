use crate::{DataItem, Persist, TasksRewardsClient};
use artifactsmmo_api_wrapper::ArtifactApi;
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
        *tasks.data.write().unwrap() = tasks.load();
        tasks
    }
}

impl Persist<HashMap<String, Arc<TaskFullSchema>>> for TasksClient {
    const PATH: &'static str = ".cache/tasks.json";

    fn load_from_api(&self) -> HashMap<String, Arc<TaskFullSchema>> {
        self.api
            .tasks
            .get_all()
            .unwrap()
            .into_iter()
            .map(|task| (task.code.clone(), Arc::new(task)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
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
