use artifactsmmo_openapi::models::{BankSchema, SimpleItemSchema};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Bank {
    pub details: RwLock<Arc<BankSchema>>,
    pub content: RwLock<Arc<Vec<SimpleItemSchema>>>,
}

impl Bank {
    pub(crate) async fn new(details: BankSchema, content: Vec<SimpleItemSchema>) -> Self {
        Self {
            details: RwLock::new(Arc::new(details)),
            content: RwLock::new(Arc::new(content)),
        }
    }

    pub async fn details(&self) -> Arc<BankSchema> {
        return self.details.read().await.clone();
    }

    pub async fn gold(&self) -> i32 {
        self.details().await.gold
    }

    pub async fn free_slots(&self) -> i32 {
        self.details().await.slots - self.content().await.len() as i32
    }

    pub async fn total_of(&self, item: &str) -> i32 {
        self.content
            .read()
            .await
            .iter()
            .find_map(|i| {
                if i.code == item {
                    Some(i.quantity)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    pub async fn content(&self) -> Arc<Vec<SimpleItemSchema>> {
        return self.content.read().await.clone();
    }

    pub async fn update_details(&self, details: BankSchema) {
        *self.details.write().await = Arc::new(details)
    }

    pub async fn update_content(&self, content: Vec<SimpleItemSchema>) {
        *self.content.write().await = Arc::new(content)
    }
}
