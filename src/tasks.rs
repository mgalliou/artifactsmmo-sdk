use crate::PersistedData;
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::TaskFullSchema;
use itertools::Itertools;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Tasks {
    data: RwLock<Vec<Arc<TaskFullSchema>>>,
    api: Arc<ArtifactApi>,
}

impl PersistedData<Vec<Arc<TaskFullSchema>>> for Tasks {
    const PATH: &'static str = ".cache/tasks.json";

    async fn data_from_api(&self) -> Vec<Arc<TaskFullSchema>> {
        self.api
            .tasks
            .all(None, None, None, None)
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

impl Tasks {
    pub(crate) async fn new(api: Arc<ArtifactApi>) -> Self {
        let tasks = Self {
            data: Default::default(),
            api,
        };
        *tasks.data.write().unwrap() = tasks.retrieve_data().await;
        tasks
    }

    pub fn all(&self) -> Vec<Arc<TaskFullSchema>> {
        self.data.read().unwrap().iter().cloned().collect_vec()
    }
}
