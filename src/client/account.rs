use artifactsmmo_api_wrapper::ArtifactApi;
use std::sync::{Arc, RwLock};

use crate::{
    character::HasCharacterData, client::{bank::BankClient, character::CharacterClient}, ClientError, ItemsClient, MapsClient, MonstersClient, NpcsClient, ResourcesClient, ServerClient, TasksClient
};

#[derive(Default, Debug)]
pub struct AccountClient {
    pub name: String,
    pub bank: Arc<BankClient>,
    pub characters: RwLock<Vec<Arc<CharacterClient>>>,
}

impl AccountClient {
    pub(crate) fn new(name: String, bank: Arc<BankClient>) -> Self {
        Self {
            name,
            bank,
            characters: RwLock::new(vec![]),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn init_characters(
        &self,
        account: Arc<AccountClient>,
        items: Arc<ItemsClient>,
        resources: Arc<ResourcesClient>,
        monsters: Arc<MonstersClient>,
        maps: Arc<MapsClient>,
        npcs: Arc<NpcsClient>,
        tasks: Arc<TasksClient>,
        server: Arc<ServerClient>,
        api: Arc<ArtifactApi>,
    ) -> Result<(), ClientError> {
        *self.characters.write().unwrap() = api
            .account
            .characters(&account.name)
            .map_err(|e| ClientError::Api(Box::new(e)))?
            .data
            .into_iter()
            .enumerate()
            .map(|(id, data)| {
                CharacterClient::new(
                    id,
                    Arc::new(RwLock::new(Arc::new(data))),
                    account.clone(),
                    items.clone(),
                    resources.clone(),
                    monsters.clone(),
                    maps.clone(),
                    npcs.clone(),
                    tasks.clone(),
                    server.clone(),
                    api.clone(),
                )
            })
            .map(Arc::new)
            .collect::<Vec<_>>();
        Ok(())
    }

    pub fn get_character_by_name(&self, name: &str) -> Option<Arc<CharacterClient>> {
        self.characters
            .read()
            .unwrap()
            .iter()
            .find(|c| c.name() == name)
            .cloned()
    }
}
