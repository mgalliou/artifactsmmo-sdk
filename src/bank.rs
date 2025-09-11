use artifactsmmo_openapi::models::{BankSchema, SimpleItemSchema};
use std::sync::{Arc, RwLock};

use crate::{ContainerSlot, ItemContainer, SlotLimited};

#[derive(Default, Debug)]
pub struct Bank {
    pub details: RwLock<Arc<BankSchema>>,
    pub content: RwLock<Arc<Vec<SimpleItemSchema>>>,
}

impl Bank {
    pub(crate) fn new(details: BankSchema, content: Vec<SimpleItemSchema>) -> Self {
        Self {
            details: RwLock::new(Arc::new(details)),
            content: RwLock::new(Arc::new(content)),
        }
    }

    pub fn details(&self) -> Arc<BankSchema> {
        return self.details.read().unwrap().clone();
    }

    pub fn gold(&self) -> u32 {
        self.details().gold
    }

    pub fn update_details(&self, details: BankSchema) {
        *self.details.write().unwrap() = Arc::new(details)
    }

    pub fn update_content(&self, content: Vec<SimpleItemSchema>) {
        *self.content.write().unwrap() = Arc::new(content)
    }
}

impl SlotLimited for Bank {}

impl ItemContainer for Bank {
    type Slot = SimpleItemSchema;

    fn content(&self) -> Arc<Vec<SimpleItemSchema>> {
        self.content.read().unwrap().clone()
    }
}

impl ContainerSlot for SimpleItemSchema {
    fn code(&self) -> &str {
        &self.code
    }
}
