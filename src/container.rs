use crate::{HasDropTable, HasQuantity};
use artifactsmmo_openapi::models::SimpleItemSchema;
use std::sync::Arc;

pub trait ContainerSlot: HasQuantity {
    fn code(&self) -> &str;
}

pub trait ItemContainer {
    type Slot: ContainerSlot;

    fn content(&self) -> Arc<Vec<Self::Slot>>;

    fn total_items(&self) -> u32 {
        self.content().iter().map(|i| i.quantity()).sum()
    }

    fn total_of(&self, item: &str) -> u32 {
        self.content()
            .iter()
            .find(|i| i.code() == item)
            .map_or(0, |i| i.quantity())
    }

    fn contains_multiple(&self, items: &[SimpleItemSchema]) -> bool {
        items.iter().all(|i| self.total_of(&i.code) >= i.quantity)
    }
}

pub trait SlotLimited: ItemContainer {
    fn free_slots(&self) -> u32 {
        self.content()
            .iter()
            .filter(|i| i.code().is_empty())
            .count() as u32
    }
}

pub trait SpaceLimited: ItemContainer {
    fn max_items(&self) -> u32;

    fn free_space(&self) -> u32 {
        self.max_items().saturating_sub(self.total_items())
    }
}

pub trait LimitedContainer: SlotLimited + SpaceLimited {
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

    fn has_space_for(&self, item: &str, quantity: u32) -> bool {
        self.has_space_for_multiple(&[SimpleItemSchema {
            code: item.to_owned(),
            quantity,
        }])
    }

    /// Checks if the `Character` inventory is full (all slots are occupied or
    /// `inventory_max_items` is reached).
    fn is_full(&self) -> bool {
        self.total_items() >= self.max_items() || self.free_slots() == 0
    }
}
