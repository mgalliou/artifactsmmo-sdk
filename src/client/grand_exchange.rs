use std::sync::{Arc, RwLock};

use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{GeOrderHistorySchema, GeOrderSchema};

#[derive(Default, Debug)]
pub struct GrandExchangeClient {
    api: Arc<ArtifactApi>,
    sell_orders: RwLock<Arc<Vec<GeOrderSchema>>>,
}

impl GrandExchangeClient {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        Self {
            api,
            sell_orders: Default::default(),
        }
    }

    pub fn sell_history(&self, code: &str) -> Option<Vec<GeOrderHistorySchema>> {
        self.api.grand_exchange.sell_history(code).ok()
    }

    pub fn sell_orders(&self) -> Arc<Vec<GeOrderSchema>> {
        self.sell_orders.read().unwrap().clone()
    }

    pub fn refresh_orders(&self) {
        *self.sell_orders.write().unwrap() =
            Arc::new(self.api.grand_exchange.sell_orders().unwrap())
    }
}
