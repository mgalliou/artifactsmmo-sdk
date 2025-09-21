use std::{result::Result, sync::Arc, vec::Vec};

use crate::{DataPage, PaginatedRequest};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        grand_exchange_api::{
            get_ge_sell_history_grandexchange_history_code_get,
            get_ge_sell_order_grandexchange_orders_id_get,
            get_ge_sell_orders_grandexchange_orders_get,
            GetGeSellHistoryGrandexchangeHistoryCodeGetError,
            GetGeSellOrderGrandexchangeOrdersIdGetError,
            GetGeSellOrdersGrandexchangeOrdersGetError,
        },
        Error,
    },
    models::{
        DataPageGeOrderHistorySchema, DataPageGeOrderSchema, GeOrderHistorySchema,
        GeOrderReponseSchema, GeOrderSchema,
    },
};

#[derive(Default, Debug)]
pub struct GrandExchangeApi {
    configuration: Arc<Configuration>,
}

impl GrandExchangeApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    pub fn sell_history(
        &self,
        code: &str,
    ) -> Result<Vec<GeOrderHistorySchema>, Error<GetGeSellHistoryGrandexchangeHistoryCodeGetError>>
    {
        SellHistoryRequest {
            configuration: &self.configuration,
            code,
        }
        .all()
    }

    pub fn sell_orders(
        &self,
    ) -> Result<Vec<GeOrderSchema>, Error<GetGeSellOrdersGrandexchangeOrdersGetError>> {
        SellOrdersRequest {
            configuration: &self.configuration,
        }
        .all()
    }

    pub fn get_sell_order(
        &self,
        id: &str,
    ) -> Result<GeOrderReponseSchema, Error<GetGeSellOrderGrandexchangeOrdersIdGetError>> {
        get_ge_sell_order_grandexchange_orders_id_get(&self.configuration, id)
    }
}

struct SellHistoryRequest<'a> {
    configuration: &'a Configuration,
    code: &'a str,
}

struct SellOrdersRequest<'a> {
    configuration: &'a Configuration,
}

impl<'a>
    PaginatedRequest<
        GeOrderHistorySchema,
        DataPageGeOrderHistorySchema,
        GetGeSellHistoryGrandexchangeHistoryCodeGetError,
    > for SellHistoryRequest<'a>
{
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageGeOrderHistorySchema, Error<GetGeSellHistoryGrandexchangeHistoryCodeGetError>>
    {
        get_ge_sell_history_grandexchange_history_code_get(
            self.configuration,
            self.code,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<GeOrderHistorySchema> for DataPageGeOrderHistorySchema {
    fn data(self) -> Vec<GeOrderHistorySchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}

impl<'a>
    PaginatedRequest<
        GeOrderSchema,
        DataPageGeOrderSchema,
        GetGeSellOrdersGrandexchangeOrdersGetError,
    > for SellOrdersRequest<'a>
{
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageGeOrderSchema, Error<GetGeSellOrdersGrandexchangeOrdersGetError>> {
        get_ge_sell_orders_grandexchange_orders_get(
            self.configuration,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<GeOrderSchema> for DataPageGeOrderSchema {
    fn data(self) -> Vec<GeOrderSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
