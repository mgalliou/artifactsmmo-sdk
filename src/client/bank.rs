use artifactsmmo_openapi::models::{BankSchema, SimpleItemSchema};
use std::sync::{Arc, RwLock};

use crate::{ItemContainer, LimitedContainer, SlotLimited};

#[derive(Default, Debug)]
pub struct BankClient {
    pub details: RwLock<Arc<BankSchema>>,
    pub content: RwLock<Arc<Vec<SimpleItemSchema>>>,
}

impl BankClient {
    pub(crate) fn new(details: BankSchema, content: Vec<SimpleItemSchema>) -> Self {
        Self {
            details: RwLock::new(Arc::new(details)),
            content: RwLock::new(Arc::new(content)),
        }
    }

    // TODO: use these methods in request handler
    pub fn update_details(&self, details: BankSchema) {
        *self.details.write().unwrap() = Arc::new(details)
    }

    pub fn update_content(&self, content: Vec<SimpleItemSchema>) {
        *self.content.write().unwrap() = Arc::new(content)
    }
}

pub trait Bank: ItemContainer + LimitedContainer + SlotLimited {
    fn details(&self) -> Arc<BankSchema>;

    fn slots(&self) -> u32 {
        self.details().slots
    }

    fn expansions(&self) -> u32 {
        self.details().expansions
    }

    fn next_expansion_cost(&self) -> u32 {
        self.details().next_expansion_cost
    }

    fn gold(&self) -> u32 {
        self.details().gold
    }
}

impl Bank for BankClient {
    fn details(&self) -> Arc<BankSchema> {
        self.details.read().unwrap().clone()
    }
}

impl ItemContainer for BankClient {
    type Slot = SimpleItemSchema;

    fn content(&self) -> Arc<Vec<SimpleItemSchema>> {
        self.content.read().unwrap().clone()
    }
}

impl SlotLimited for BankClient {
    fn free_slots(&self) -> u32 {
        self.details()
            .slots
            .saturating_sub(self.content().len() as u32)
    }
}

impl LimitedContainer for BankClient {
    fn is_full(&self) -> bool {
        self.free_slots() == 0
    }

    fn has_room_for_multiple(&self, items: &[SimpleItemSchema]) -> bool {
        let mut free_slot = self.free_slots();
        for item in items.iter() {
            if free_slot < 1 {
                return false;
            }
            if self.total_of(&item.code) < 1 {
                free_slot -= 1
            }
        }
        true
    }

    fn has_room_for_drops_from<H: crate::DropsItems>(&self, entity: &H) -> bool {
        self.free_slots() >= entity.average_drop_slots()
    }
}
