use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        npcs_api::{get_all_npcs_npcs_details_get, GetAllNpcsNpcsDetailsGetError},
        Error,
    },
    models::{DataPageNpcSchema, NpcSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct NpcsApi {
    configuration: Arc<Configuration>,
}

impl NpcsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl PaginatedApi<NpcSchema, DataPageNpcSchema, GetAllNpcsNpcsDetailsGetError> for NpcsApi {
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageNpcSchema, Error<GetAllNpcsNpcsDetailsGetError>> {
        get_all_npcs_npcs_details_get(
            &self.configuration,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<NpcSchema> for DataPageNpcSchema {
    fn data(self) -> Vec<NpcSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
