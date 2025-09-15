use std::sync::Arc;

use crate::client::{bank::BankClient, character::CharacterClient};

#[derive(Default, Debug)]
pub struct AccountClient {
    pub name: String,
    pub bank: Arc<BankClient>,
    pub characters: Vec<Arc<CharacterClient>>,
}

impl AccountClient {
    pub(crate) fn new(
        name: String,
        bank: Arc<BankClient>,
        characters: Vec<Arc<CharacterClient>>,
    ) -> Self {
        Self {
            name,
            bank,
            characters,
        }
    }
}
