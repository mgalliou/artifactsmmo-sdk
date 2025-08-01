use std::sync::Arc;

use crate::{Bank, Character};

#[derive(Default, Debug)]
pub struct Account {
    pub name: String,
    pub bank: Arc<Bank>,
    pub characters: Vec<Arc<Character>>,
}

impl Account {
    pub(crate) fn new(name: String, bank: Arc<Bank>, characters: Vec<Arc<Character>>) -> Self {
        Self {
            name,
            bank,
            characters,
        }
    }
}
