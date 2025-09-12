use crate::{
    HasDropTable,
    container::{ContainerSlot, ItemContainer, LimitedContainer, SlotLimited, SpaceLimited},
};
use artifactsmmo_openapi::models::{CharacterSchema, InventorySlot, SimpleItemSchema};
use itertools::Itertools;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct InventoryClient {
    data: Arc<CharacterSchema>,
}

impl InventoryClient {
    pub fn new(data: Arc<CharacterSchema>) -> Self {
        Self { data }
    }
}

pub trait Inventory: LimitedContainer + SlotLimited + SpaceLimited {}

impl Inventory for InventoryClient {}

impl ItemContainer for InventoryClient {
    type Slot = InventorySlot;

    fn content(&self) -> Arc<Vec<InventorySlot>> {
        Arc::new(self.data.inventory.iter().flatten().cloned().collect_vec())
    }
}

impl SpaceLimited for InventoryClient {
    fn max_items(&self) -> u32 {
        self.data.inventory_max_items as u32
    }
}

impl SlotLimited for InventoryClient {
    fn free_slots(&self) -> u32 {
        self.content()
            .iter()
            .filter(|i| i.code().is_empty())
            .count() as u32
    }
}

impl LimitedContainer for InventoryClient {
    fn is_full(&self) -> bool {
        self.total_items() >= self.max_items() || self.free_slots() == 0
    }

    fn has_space_for_multiple(&self, items: &[SimpleItemSchema]) -> bool {
        let mut free_slot = self.free_slots();
        let mut free_space = self.free_space();
        for item in items.iter() {
            if free_slot < 1 || free_space < item.quantity {
                return false;
            }
            if self.total_of(&item.code) < 1 {
                free_slot -= 1
            }
            free_space -= item.quantity
        }
        true
    }

    fn has_space_for_drops_from<H: HasDropTable>(&self, entity: &H) -> bool {
        self.free_slots() >= entity.average_drop_slots()
            && self.free_space() >= entity.average_drop_quantity()
    }
}

impl ContainerSlot for InventorySlot {
    fn code(&self) -> &str {
        &self.code
    }
}
