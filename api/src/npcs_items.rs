use crate::{DataPage, PaginatedRequest};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        npcs_api::{get_all_npcs_items_npcs_items_get, GetAllNpcsItemsNpcsItemsGetError},
        Error,
    },
    models::{DataPageNpcItem, NpcItem},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct NpcsItemsApi {
    configuration: Arc<Configuration>,
}

impl NpcsItemsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl PaginatedRequest<NpcItem, DataPageNpcItem, GetAllNpcsItemsNpcsItemsGetError> for NpcsItemsApi {
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageNpcItem, Error<GetAllNpcsItemsNpcsItemsGetError>> {
        get_all_npcs_items_npcs_items_get(
            &self.configuration,
            None,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<NpcItem> for DataPageNpcItem {
    fn data(self) -> Vec<NpcItem> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
