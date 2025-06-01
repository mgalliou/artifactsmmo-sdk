use crate::char::CharacterData;
use artifactsmmo_api_wrapper::ArtifactApi;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default)]
pub struct Account {
    pub name: String,
    characters_data: Arc<HashMap<usize, CharacterData>>,
}

impl Account {
    pub(crate) fn new(api: &Arc<ArtifactApi>, name: String) -> Self {
        let characters_data = Arc::new(
            api.account
                .characters(&name)
                .unwrap()
                .data
                .into_iter()
                .enumerate()
                .map(|(id, data)| (id, Arc::new(RwLock::new(Arc::new(data)))))
                .collect::<_>(),
        );
        Self {
            name,
            characters_data,
        }
    }

    pub(crate) fn characters_data(&self) -> Arc<HashMap<usize, CharacterData>> {
        self.characters_data.clone()
    }
}
