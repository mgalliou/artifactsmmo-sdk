use crate::{Collection, Data, PersistedData};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::{RewardsSchema, TaskFullSchema};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard},
};

#[derive(Default, Debug)]
pub struct Tasks {
    data: RwLock<HashMap<String, Arc<TaskFullSchema>>>,
    api: Arc<ArtifactApi>,
}

impl PersistedData<HashMap<String, Arc<TaskFullSchema>>> for Tasks {
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

impl Data for Tasks {
    type Item = Arc<TaskFullSchema>;

    fn data(&self) -> RwLockReadGuard<'_, HashMap<String, Self::Item>> {
        self.data.read().unwrap()
    }
}

impl Collection for Tasks {}

impl Tasks {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let tasks = Self {
            data: Default::default(),
            api,
        };
        *tasks.data.write().unwrap() = tasks.retrieve_data();
        tasks
    }
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
