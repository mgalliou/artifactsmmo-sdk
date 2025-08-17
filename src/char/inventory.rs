use crate::items::{ItemSchemaExt, Items};
use artifactsmmo_openapi::models::{InventorySlot, ItemSchema};
use itertools::Itertools;
use std::sync::Arc;

use super::CharacterData;

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

    pub fn has_space_for(&self, item: &str, quantity: i32) -> bool {
        if self.total_of(item) > 0 {
            self.free_space() >= quantity
        } else {
            self.free_slots() > 0
        }
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

    pub fn consumable_food(&self) -> Vec<Arc<ItemSchema>> {
        self.data
            .read()
            .unwrap()
            .inventory
            .iter()
            .flatten()
            .filter_map(|i| {
                self.items
                    .get(&i.code)
                    .filter(|i| i.is_consumable_at(self.data.read().unwrap().level))
            })
            .collect_vec()
    }
}
