use crate::{
    Account, Bank, Character, Events, Items, Maps, Monsters, Resources, Server, Tasks, TasksRewards,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use std::sync::Arc;

pub struct Client {
    pub account: Arc<Account>,
    pub server: Arc<Server>,
    pub events: Arc<Events>,
    pub resources: Arc<Resources>,
    pub monsters: Arc<Monsters>,
    pub items: Arc<Items>,
    pub tasks: Arc<Tasks>,
    pub tasks_rewards: Arc<TasksRewards>,
    pub maps: Arc<Maps>,
    pub bank: Arc<Bank>,
    pub characters: Vec<Arc<Character>>,
}

impl Client {
    pub fn new(url: String, account_name: String, token: String) -> Self {
        let api = Arc::new(ArtifactApi::new(url, token.clone()));
        let account = Arc::new(Account::new(&api, account_name));
        let bank = Arc::new(Bank::new(
            *api.bank.details().unwrap().data,
            api.bank.items(None).unwrap(),
        ));
        let events = Arc::new(Events::new(api.clone()));
        let resources = Arc::new(Resources::new(api.clone(), events.clone()));
        let monsters = Arc::new(Monsters::new(api.clone(), events.clone()));
        let tasks_rewards = Arc::new(TasksRewards::new(api.clone()));
        let items = Arc::new(Items::new(
            api.clone(),
            resources.clone(),
            monsters.clone(),
            tasks_rewards.clone(),
        ));
        let maps = Arc::new(Maps::new(&api, events.clone()));
        let server = Arc::new(Server::new(api.clone()));
        let characters = account
            .characters_data()
            .iter()
            .map(|(id, data)| {
                Character::new(
                    *id,
                    data.clone(),
                    bank.clone(),
                    items.clone(),
                    resources.clone(),
                    monsters.clone(),
                    maps.clone(),
                    server.clone(),
                    api.clone(),
                )
            })
            .map(Arc::new)
            .collect::<_>();
        Self {
            account,
            bank,
            items,
            tasks: Arc::new(Tasks::new(api.clone())),
            monsters,
            resources,
            characters,
            server,
            events,
            tasks_rewards,
            maps,
        }
    }
}
