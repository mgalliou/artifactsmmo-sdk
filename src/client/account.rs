use crate::{
    ClientError, ItemsClient, MapsClient, MonstersClient, NpcsClient, ResourcesClient,
    ServerClient, TasksClient,
    character::HasCharacterData,
    client::{bank::BankClient, character::CharacterClient},
    grand_exchange::GrandExchangeClient,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use itertools::Itertools;
use std::sync::{Arc, RwLock};

#[derive(Default, Debug)]
pub struct AccountClient {
    pub name: String,
    pub bank: Arc<BankClient>,
    characters: RwLock<Vec<Arc<CharacterClient>>>,
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
        grand_exchange: Arc<GrandExchangeClient>,
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
                    grand_exchange.clone(),
                    server.clone(),
                    api.clone(),
                )
            })
            .map(Arc::new)
            .collect_vec();
        Ok(())
    }

    pub fn characters(&self) -> Vec<Arc<CharacterClient>> {
        self.characters.read().unwrap().iter().cloned().collect()
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
