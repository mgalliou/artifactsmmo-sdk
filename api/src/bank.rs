use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        my_account_api::{
            get_bank_details_my_bank_get, get_bank_items_my_bank_items_get,
            GetBankDetailsMyBankGetError, GetBankItemsMyBankItemsGetError,
        },
        Error,
    },
    models::{BankResponseSchema, DataPageSimpleItemSchema, SimpleItemSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct BankApi {
    configuration: Arc<Configuration>,
}

impl BankApi {
    pub fn new(configuration: Arc<Configuration>) -> Self {
        BankApi { configuration }
    }

    pub fn details(&self) -> Result<BankResponseSchema, Error<GetBankDetailsMyBankGetError>> {
        get_bank_details_my_bank_get(&self.configuration)
    }
}

impl PaginatedApi<SimpleItemSchema, DataPageSimpleItemSchema, GetBankItemsMyBankItemsGetError>
    for BankApi
{
    fn api_call(
        &self,
        current_page: i32,
    ) -> Result<DataPageSimpleItemSchema, Error<GetBankItemsMyBankItemsGetError>> {
        get_bank_items_my_bank_items_get(&self.configuration, None, Some(current_page), Some(100))
    }
}

impl DataPage<SimpleItemSchema> for DataPageSimpleItemSchema {
    fn data(self) -> Vec<SimpleItemSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<i32>> {
        self.pages
    }
}
