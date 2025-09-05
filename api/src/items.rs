use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        items_api::{
            get_all_items_items_get, get_item_items_code_get, GetAllItemsItemsGetError,
            GetItemItemsCodeGetError,
        },
        Error,
    },
    models::{DataPageItemSchema, ItemResponseSchema, ItemSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct ItemsApi {
    configuration: Arc<Configuration>,
}

impl ItemsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    pub fn get(&self, code: &str) -> Result<ItemResponseSchema, Error<GetItemItemsCodeGetError>> {
        get_item_items_code_get(&self.configuration, code)
    }
}

impl PaginatedApi<ItemSchema, DataPageItemSchema, GetAllItemsItemsGetError> for ItemsApi {
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageItemSchema, Error<GetAllItemsItemsGetError>> {
        get_all_items_items_get(
            &self.configuration,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<ItemSchema> for DataPageItemSchema {
    fn data(self) -> Vec<ItemSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
