use std::sync::Arc;

use crate::{BankClient, Character};

#[derive(Default, Debug)]
pub struct Account {
    pub name: String,
    pub bank: Arc<BankClient>,
    pub characters: Vec<Arc<Character>>,
}

impl Account {
    pub(crate) fn new(name: String, bank: Arc<BankClient>, characters: Vec<Arc<Character>>) -> Self {
        Self {
            name,
            bank,
            characters,
        }
    }
}
