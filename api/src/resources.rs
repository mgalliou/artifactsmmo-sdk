use crate::{DataPage, Paginate};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        resources_api::{get_all_resources_resources_get, GetAllResourcesResourcesGetError},
        Error,
    },
    models::{DataPageResourceSchema, ResourceSchema},
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

    pub fn get_all(&self) -> Result<Vec<ResourceSchema>, Error<GetAllResourcesResourcesGetError>> {
        ResourcesRequest {
            configuration: &self.configuration,
        }
        .send()
    }
}

struct ResourcesRequest<'a> {
    configuration: &'a Configuration,
}

impl<'a> Paginate for ResourcesRequest<'a> {
    type Data = ResourceSchema;
    type Page = DataPageResourceSchema;
    type Error = GetAllResourcesResourcesGetError;

    fn request_page(&self, page: u32) -> Result<Self::Page, Error<Self::Error>> {
        get_all_resources_resources_get(
            self.configuration,
            None,
            None,
            None,
            None,
            Some(page),
            Some(100),
        )
    }
}

impl DataPage<ResourceSchema> for DataPageResourceSchema {
    fn data(self) -> Vec<ResourceSchema> {
        self.data
    }

    fn pages(&self) -> Option<u32> {
        self.pages
    }
}
