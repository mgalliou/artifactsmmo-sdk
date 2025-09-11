use crate::container::{ContainerSlot, ItemContainer, LimitedContainer, SlotLimited, SpaceLimited};
use artifactsmmo_openapi::models::{CharacterSchema, InventorySlot};
use itertools::Itertools;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Inventory {
    data: Arc<CharacterSchema>,
}

impl Inventory {
    pub fn new(data: Arc<CharacterSchema>) -> Self {
        Self { data }
    }
}

impl ItemContainer for Inventory {
    type Slot = InventorySlot;

    fn content(&self) -> Arc<Vec<InventorySlot>> {
        Arc::new(self.data.inventory.iter().flatten().cloned().collect_vec())
    }
}

impl SpaceLimited for Inventory {
    fn max_items(&self) -> u32 {
        self.data.inventory_max_items as u32
    }
}

impl ContainerSlot for InventorySlot {
    fn code(&self) -> &str {
        &self.code
    }
}

impl LimitedContainer for Inventory {}
impl SlotLimited for Inventory {}
