use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        tasks_api::{
            get_all_tasks_rewards_tasks_rewards_get, get_all_tasks_tasks_list_get,
            GetAllTasksRewardsTasksRewardsGetError, GetAllTasksTasksListGetError,
        },
        Error,
    },
    models::{DataPageTaskFullSchema, DropRateSchema, TaskFullSchema},
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

    pub fn rewards(
        &self,
    ) -> Result<Vec<DropRateSchema>, Error<GetAllTasksRewardsTasksRewardsGetError>> {
        let mut drops: Vec<DropRateSchema> = vec![];
        let mut current_page = 1;
        let mut finished = false;
        while !finished {
            let resp = get_all_tasks_rewards_tasks_rewards_get(
                &self.configuration,
                Some(current_page),
                Some(100),
            );
            match resp {
                Ok(resp) => {
                    drops.extend(resp.data);
                    if let Some(Some(pages)) = resp.pages {
                        if current_page >= pages {
                            finished = true
                        }
                        current_page += 1;
                    } else {
                        // No pagination information, assume single page
                        finished = true
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(drops)
    }
}

impl PaginatedApi<TaskFullSchema, DataPageTaskFullSchema, GetAllTasksTasksListGetError>
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
