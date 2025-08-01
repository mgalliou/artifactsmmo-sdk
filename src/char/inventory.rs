use crate::items::{ItemSchemaExt, Items};
use artifactsmmo_openapi::models::{InventorySlot, ItemSchema};
use futures::{stream, StreamExt, TryStreamExt};
use itertools::Itertools;
use std::sync::Arc;

use super::CharacterData;

pub struct Inventory {
    data: CharacterData,
    items: Arc<Items>,
}

impl Inventory {
    pub fn new(data: CharacterData, items: Arc<Items>) -> Self {
        Self { data, items }
    }

    /// Returns the amount of item in the `Character` inventory.
    pub async fn total_items(&self) -> i32 {
        self.data
            .read()
            .await
            .inventory
            .iter()
            .flatten()
            .map(|i| i.quantity)
            .sum()
    }

    /// Returns the maximum number of item the inventory can contain.
    pub async fn max_items(&self) -> i32 {
        self.data.read().await.inventory_max_items
    }

    /// Returns the free spaces in the `Character` inventory.
    pub async fn free_space(&self) -> i32 {
        self.max_items().await - self.total_items().await
    }

    /// Checks if the `Character` inventory is full (all slots are occupied or
    /// `inventory_max_items` is reached).
    pub async fn is_full(&self) -> bool {
        self.total_items().await >= self.max_items().await
            || self
                .data
                .read()
                .await
                .inventory
                .iter()
                .flatten()
                .all(|s| s.quantity > 0)
    }

    /// Returns the amount of the given item `code` in the `Character` inventory.
    pub async fn total_of(&self, item: &str) -> i32 {
        self.data
            .read()
            .await
            .inventory
            .iter()
            .flatten()
            .find(|i| i.code == item)
            .map_or(0, |i| i.quantity)
    }

    pub async fn contains_mats_for(&self, item: &str, quantity: i32) -> bool {
        stream::iter(self.items.mats_of(item).await)
            .all(async |m| self.total_of(&m.code).await >= m.quantity * quantity)
            .await
    }

    pub async fn consumable_food(&self) -> Vec<Arc<ItemSchema>> {
        let data = self.data.read().await;

        stream::iter(data.inventory.iter().flatten())
            .filter_map(async |slot| {
                self.items
                    .get(&slot.code)
                    .await
                    .filter(|i| i.is_consumable_at(data.level))
            })
            .collect::<Vec<_>>()
            .await
    }
}
