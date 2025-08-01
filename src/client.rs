
use crate::{
    Account, Bank, Character, Events, Items, Maps, Monsters, Resources, Server, Tasks, TasksRewards,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    pub async fn new(url: String, account_name: String, token: String) -> Self {
        let api = Arc::new(ArtifactApi::new(url, token.clone()).await);
        let bank = Arc::new(Bank::new(
            *api.bank.details().await.unwrap().data,
            api.bank.items(None).await.unwrap(),
        ).await);
        let events = Arc::new(Events::new(api.clone()).await);
        let resources = Arc::new(Resources::new(api.clone(), events.clone()).await);
        let monsters = Arc::new(Monsters::new(api.clone(), events.clone()).await);
        let tasks_rewards = Arc::new(TasksRewards::new(api.clone()).await);
        let items = Arc::new(Items::new(
            api.clone(),
            resources.clone(),
            monsters.clone(),
            tasks_rewards.clone(),
        ).await);
        let maps = Arc::new(Maps::new(&api, events.clone()).await);
        let server = Arc::new(Server::new(api.clone()).await);
        let characters_data = api
            .account
            .characters(&account_name)
            .await
            .unwrap()
            .data;
        let mut characters = Vec::new();
        for (id, data) in characters_data.into_iter().enumerate() {
            let character = Character::new(
                id,
                Arc::new(RwLock::new(Arc::new(data))),
                bank.clone(),
                items.clone(),
                resources.clone(),
                monsters.clone(),
                maps.clone(),
                server.clone(),
                api.clone(),
            );
            characters.push(Arc::new(character));
        }
        let account = Arc::new(Account::new(account_name, bank.clone(), characters));
        Self {
            account,
            items,
            monsters,
            resources,
            server,
            events,
            tasks: Arc::new(Tasks::new(api.clone()).await),
            tasks_rewards,
            maps,
        }
    }
}
