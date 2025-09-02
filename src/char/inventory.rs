use super::CharacterData;
use crate::{HasDropTable, items::Items};
use artifactsmmo_openapi::models::{InventorySlot, SimpleItemSchema};
use itertools::Itertools;
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Inventory {
    data: CharacterData,
    items: Arc<Items>,
}

impl Inventory {
    pub fn new(data: CharacterData, items: Arc<Items>) -> Self {
        Self { data, items }
    }

    pub fn content(&self) -> Vec<InventorySlot> {
        self.data
            .read()
            .unwrap()
            .inventory
            .iter()
            .flatten()
            .cloned()
            .collect_vec()
    }

    /// Returns the amount of item in the `Character` inventory.
    pub fn total_items(&self) -> i32 {
        self.data
            .read()
            .unwrap()
            .inventory
            .iter()
            .flatten()
            .map(|i| i.quantity)
            .sum()
    }

    /// Returns the maximum number of item the inventory can contain.
    pub fn max_items(&self) -> i32 {
        self.data.read().unwrap().inventory_max_items
    }

    /// Returns the free spaces in the `Character` inventory.
    pub fn free_space(&self) -> i32 {
        self.max_items() - self.total_items()
    }

    pub fn free_slots(&self) -> usize {
        self.data
            .read()
            .unwrap()
            .inventory
            .iter()
            .flatten()
            .filter(|i| i.code.is_empty())
            .count()
    }

    pub fn has_space_for_multiple(&self, items: &[SimpleItemSchema]) -> bool {
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

    pub fn has_space_for_drops_from<H: HasDropTable>(&self, entity: &H) -> bool {
        self.free_slots() >= entity.average_drop_slots() as usize
            && self.free_space() >= entity.average_drop_quantity()
    }

    pub fn has_space_for(&self, item: &str, quantity: i32) -> bool {
        self.has_space_for_multiple(&[SimpleItemSchema {
            code: item.to_owned(),
            quantity,
        }])
    }

    /// Checks if the `Character` inventory is full (all slots are occupied or
    /// `inventory_max_items` is reached).
    pub fn is_full(&self) -> bool {
        self.total_items() >= self.max_items() || self.free_slots() == 0
    }

    /// Returns the amount of the given item `code` in the `Character` inventory.
    pub fn total_of(&self, item: &str) -> i32 {
        self.data
            .read()
            .unwrap()
            .inventory
            .iter()
            .flatten()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }

    pub fn contains_mats_for(&self, item: &str, quantity: i32) -> bool {
        self.items
            .mats_of(item)
            .iter()
            .all(|m| self.total_of(&m.code) >= m.quantity * quantity)
    }
}
