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
    fn free_slots(&self) -> u32;
}

pub trait SpaceLimited: ItemContainer {
    fn max_items(&self) -> u32;

    fn free_space(&self) -> u32 {
        self.max_items().saturating_sub(self.total_items())
    }
}

pub trait LimitedContainer {
    fn is_full(&self) -> bool;
    fn has_space_for_multiple(&self, items: &[SimpleItemSchema]) -> bool;
    fn has_space_for_drops_from<H: HasDropTable>(&self, entity: &H) -> bool;

    fn has_space_for(&self, item: &str, quantity: u32) -> bool {
        self.has_space_for_multiple(&[SimpleItemSchema {
            code: item.to_owned(),
            quantity,
        }])
    }
}
