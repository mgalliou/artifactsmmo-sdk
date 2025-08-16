use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        resources_api::{
            get_all_resources_resources_get, get_resource_resources_code_get,
            GetAllResourcesResourcesGetError, GetResourceResourcesCodeGetError,
        },
        Error,
    },
    models::{DataPageResourceSchema, ResourceResponseSchema, ResourceSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct ResourcesApi {
    configuration: Arc<Configuration>,
}

impl ResourcesApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    pub fn get(
        &self,
        code: &str,
    ) -> Result<ResourceResponseSchema, Error<GetResourceResourcesCodeGetError>> {
        get_resource_resources_code_get(&self.configuration, code)
    }
}

impl PaginatedApi<ResourceSchema, DataPageResourceSchema, GetAllResourcesResourcesGetError>
    for ResourcesApi
{
    fn api_call(
        &self,
        current_page: i32,
    ) -> Result<DataPageResourceSchema, Error<GetAllResourcesResourcesGetError>> {
        get_all_resources_resources_get(
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

impl DataPage<ResourceSchema> for DataPageResourceSchema {
    fn data(self) -> Vec<ResourceSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<i32>> {
        self.pages
    }
}
