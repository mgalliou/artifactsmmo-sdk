use crate::PersistedData;
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::DropRateSchema;
use itertools::Itertools;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct TasksRewards {
    data: RwLock<Vec<Arc<DropRateSchema>>>,
    api: Arc<ArtifactApi>,
}

impl PersistedData<Vec<Arc<DropRateSchema>>> for TasksRewards {
    const PATH: &'static str = ".cache/tasks_rewards.json";

    async fn data_from_api(&self) -> Vec<Arc<DropRateSchema>> {
        self.api
            .tasks
            .rewards()
            .await
            .unwrap()
            .into_iter()
            .map(Arc::new)
            .collect_vec()
    }

    async fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api().await;
    }
}

impl TasksRewards {
    pub(crate) async fn new(api: Arc<ArtifactApi>) -> Self {
        let rewards = Self {
            data: Default::default(),
            api,
        };
        *rewards.data.write().unwrap() = rewards.retrieve_data().await;
        rewards
    }

    pub fn all(&self) -> Vec<Arc<DropRateSchema>> {
        self.data.read().unwrap().iter().cloned().collect_vec()
    }
}
