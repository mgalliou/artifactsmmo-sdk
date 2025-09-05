use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        tasks_api::{
            get_all_tasks_rewards_tasks_rewards_get, GetAllTasksRewardsTasksRewardsGetError,
        },
        Error,
    },
    models::{DataPageDropRateSchema, DropRateSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct TasksRewardApi {
    configuration: Arc<Configuration>,
}

impl TasksRewardApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl PaginatedApi<DropRateSchema, DataPageDropRateSchema, GetAllTasksRewardsTasksRewardsGetError>
    for TasksRewardApi
{
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageDropRateSchema, Error<GetAllTasksRewardsTasksRewardsGetError>> {
        get_all_tasks_rewards_tasks_rewards_get(&self.configuration, Some(current_page), Some(100))
    }
}

impl DataPage<DropRateSchema> for DataPageDropRateSchema {
    fn data(self) -> Vec<DropRateSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
