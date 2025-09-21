use crate::{DataPage, PaginatedRequest};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        tasks_api::{get_all_tasks_tasks_list_get, GetAllTasksTasksListGetError},
        Error,
    },
    models::{DataPageTaskFullSchema, TaskFullSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct TasksApi {
    configuration: Arc<Configuration>,
}

impl TasksApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl PaginatedRequest<TaskFullSchema, DataPageTaskFullSchema, GetAllTasksTasksListGetError>
    for TasksApi
{
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageTaskFullSchema, Error<GetAllTasksTasksListGetError>> {
        get_all_tasks_tasks_list_get(
            &self.configuration,
            None,
            None,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<TaskFullSchema> for DataPageTaskFullSchema {
    fn data(self) -> Vec<TaskFullSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
