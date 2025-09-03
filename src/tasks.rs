use crate::PersistedData;
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::{RewardsSchema, TaskFullSchema};
use itertools::Itertools;
use std::sync::{Arc, RwLock};

#[derive(Default, Debug)]
pub struct Tasks {
    data: RwLock<Vec<Arc<TaskFullSchema>>>,
    api: Arc<ArtifactApi>,
}

impl PersistedData<Vec<Arc<TaskFullSchema>>> for Tasks {
    const PATH: &'static str = ".cache/tasks.json";

    fn data_from_api(&self) -> Vec<Arc<TaskFullSchema>> {
        self.api
            .tasks
            .all()
            .unwrap()
            .into_iter()
            .map(Arc::new)
            .collect_vec()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl Tasks {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let tasks = Self {
            data: Default::default(),
            api,
        };
        *tasks.data.write().unwrap() = tasks.retrieve_data();
        tasks
    }

    pub fn all(&self) -> Vec<Arc<TaskFullSchema>> {
        self.data.read().unwrap().iter().cloned().collect_vec()
    }
}

trait TaskFullSchemaExt {
    fn rewards_quantity(&self) -> i32 {
        self.rewards().items.iter().map(|i| i.quantity).sum()
    }

    fn rewards_slots(&self) -> usize {
        self.rewards().items.len()
    }

    fn rewards(&self) -> &RewardsSchema;
}

impl TaskFullSchemaExt for TaskFullSchema {
    fn rewards(&self) -> &RewardsSchema {
        self.rewards.as_ref()
    }
}
