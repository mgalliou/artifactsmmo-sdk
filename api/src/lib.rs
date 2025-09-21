use artifactsmmo_openapi::apis::{configuration::Configuration, Error};
use std::sync::Arc;

pub use account::AccountApi;
pub use bank::BankApi;
pub use characters::CharactersApi;
pub use events::EventsApi;
pub use items::ItemsApi;
pub use maps::MapsApi;
pub use monsters::MonstersApi;
pub use my_characters::MyCharacterApi;
pub use npcs::NpcsApi;
pub use resources::ResourcesApi;
pub use server::ServerApi;
pub use tasks::TasksApi;

use crate::{
    active_events::ActiveEventsApi, grand_exchange::GrandExchangeApi, npcs_items::NpcsItemsApi,
    tasks_reward::TasksRewardApi,
};

pub mod account;
pub mod active_events;
pub mod bank;
pub mod characters;
pub mod events;
pub mod grand_exchange;
pub mod items;
pub mod maps;
pub mod monsters;
pub mod my_characters;
pub mod npcs;
pub mod npcs_items;
pub mod resources;
pub mod server;
pub mod tasks;
pub mod tasks_reward;

#[derive(Default, Debug)]
pub struct ArtifactApi {
    pub account: AccountApi,
    pub active_events: ActiveEventsApi,
    pub bank: BankApi,
    pub character: CharactersApi,
    pub events: EventsApi,
    pub items: ItemsApi,
    pub maps: MapsApi,
    pub monsters: MonstersApi,
    pub my_character: MyCharacterApi,
    pub npcs: NpcsApi,
    pub npcs_items: NpcsItemsApi,
    pub resources: ResourcesApi,
    pub server: ServerApi,
    pub tasks: TasksApi,
    pub tasks_reward: TasksRewardApi,
    pub grand_exchange: GrandExchangeApi,
}

impl ArtifactApi {
    pub fn new(base_path: String, token: String) -> Self {
        let conf = Arc::new({
            let mut c = Configuration::new();
            c.base_path = base_path;
            c
        });
        let auth_conf = Arc::new({
            let mut c = (*conf.clone()).clone();
            c.bearer_access_token = Some(token);
            c
        });
        Self {
            account: AccountApi::new(auth_conf.clone()),
            bank: BankApi::new(auth_conf.clone()),
            character: CharactersApi::new(conf.clone()),
            events: EventsApi::new(conf.clone()),
            active_events: ActiveEventsApi::new(conf.clone()),
            items: ItemsApi::new(conf.clone()),
            maps: MapsApi::new(conf.clone()),
            monsters: MonstersApi::new(conf.clone()),
            my_character: MyCharacterApi::new(auth_conf.clone()),
            resources: ResourcesApi::new(conf.clone()),
            tasks: TasksApi::new(conf.clone()),
            tasks_reward: TasksRewardApi::new(conf.clone()),
            server: ServerApi::new(conf.clone()),
            npcs: NpcsApi::new(conf.clone()),
            npcs_items: NpcsItemsApi::new(conf.clone()),
            grand_exchange: GrandExchangeApi::new(conf.clone()),
        }
    }
}

pub trait PaginatedRequest<T, P, E>
where
    P: DataPage<T>,
{
    fn all(&self) -> Result<Vec<T>, Error<E>> {
        let mut npcs: Vec<T> = vec![];
        let mut current_page = 1;
        let mut finished = false;
        while !finished {
            let resp = self.api_call(current_page);
            match resp {
                Ok(resp) => {
                    if let Some(Some(pages)) = resp.pages() {
                        if current_page >= pages {
                            finished = true
                        }
                        current_page += 1;
                    } else {
                        // No pagination information, assume single page
                        finished = true
                    }
                    npcs.extend(resp.data());
                }
                Err(e) => return Err(e),
            }
        }
        Ok(npcs)
    }
    fn api_call(&self, current_page: u32) -> Result<P, Error<E>>;
}

pub trait DataPage<T> {
    fn data(self) -> Vec<T>;
    fn pages(&self) -> Option<Option<u32>>;
}
