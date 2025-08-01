use crate::{
    Account, Bank, Character, Events, Items, Maps, Monsters, Resources, Server, Tasks, TasksRewards,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use std::sync::{Arc, RwLock};

#[derive(Default, Debug)]
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
}

impl Client {
    pub fn new(url: String, account_name: String, token: String) -> Self {
        let api = Arc::new(ArtifactApi::new(url, token));
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
        let characters = api
            .account
            .characters(&account_name)
            .unwrap()
            .data
            .into_iter()
            .enumerate()
            .map(|(id, data)| {
                Character::new(
                    id,
                    Arc::new(RwLock::new(Arc::new(data))),
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
        let account = Arc::new(Account::new(account_name, bank.clone(), characters));
        Self {
            account,
            items,
            monsters,
            resources,
            server,
            events,
            tasks: Arc::new(Tasks::new(api.clone())),
            tasks_rewards,
            maps,
        }
    }
}
