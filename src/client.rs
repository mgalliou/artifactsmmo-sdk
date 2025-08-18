use crate::{
    Account, Bank, Character, Events, Items, Maps, Monsters, Resources, Server, Tasks,
    TasksRewards, error::ClientError, npcs::Npcs, npcs_items::NpcsItems,
};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use std::{
    sync::{Arc, RwLock},
    thread,
};

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
    pub npcs: Arc<Npcs>,
}

impl Client {
    pub fn new(url: String, account_name: String, token: String) -> Result<Self, ClientError> {
        let api = Arc::new(ArtifactApi::new(url, token));

        let (bank_res, events, tasks_rewards, server, tasks) = thread::scope(|s| {
            let api_clone = api.clone();
            let bank_handle = s.spawn(move || {
                let bank_details = api_clone
                    .bank
                    .details()
                    .map_err(|e| ClientError::Api(Box::new(e)))?;
                let bank_items = api_clone
                    .bank
                    .all()
                    .map_err(|e| ClientError::Api(Box::new(e)))?;
                Ok(Arc::new(Bank::new(*bank_details.data, bank_items)))
            });

            let api_clone = api.clone();
            let events_handle = s.spawn(move || Arc::new(Events::new(api_clone.clone())));

            let api_clone = api.clone();
            let tasks_rewards_handle =
                s.spawn(move || Arc::new(TasksRewards::new(api_clone.clone())));

            let api_clone = api.clone();
            let server_handle = s.spawn(move || Arc::new(Server::new(api_clone.clone())));

            let api_clone = api.clone();
            let tasks_handle = s.spawn(move || Arc::new(Tasks::new(api_clone.clone())));

            (
                bank_handle.join().unwrap(),
                events_handle.join().unwrap(),
                tasks_rewards_handle.join().unwrap(),
                server_handle.join().unwrap(),
                tasks_handle.join().unwrap(),
            )
        });

        let bank: Arc<Bank> = bank_res?;

        let (resources, monsters, maps) = thread::scope(|s| {
            let api_clone = api.clone();
            let events_clone = events.clone();
            let resources_handle =
                s.spawn(move || Arc::new(Resources::new(api_clone.clone(), events_clone)));

            let api_clone = api.clone();
            let events_clone = events.clone();
            let monsters_handle =
                s.spawn(move || Arc::new(Monsters::new(api_clone.clone(), events_clone)));

            let api_clone = api.clone();
            let events_clone = events.clone();
            let maps_handle = s.spawn(move || Arc::new(Maps::new(&api_clone, events_clone)));

            (
                resources_handle.join().unwrap(),
                monsters_handle.join().unwrap(),
                maps_handle.join().unwrap(),
            )
        });

        let npcs = Arc::new(Npcs::new(
            api.clone(),
            Arc::new(NpcsItems::new(api.clone())),
        ));

        let items = Arc::new(Items::new(
            api.clone(),
            resources.clone(),
            monsters.clone(),
            tasks_rewards.clone(),
            npcs.clone(),
        ));

        let characters = thread::scope(|s| {
            let api_clone = api.clone();
            let bank_clone = bank.clone();
            let items_clone = items.clone();
            let resources_clone = resources.clone();
            let monsters_clone = monsters.clone();
            let maps_clone = maps.clone();
            let npcs_clone = npcs.clone();
            let server_clone = server.clone();
            let account_name_clone = account_name.clone();

            s.spawn(move || {
                Ok(api_clone
                    .account
                    .characters(&account_name_clone)
                    .map_err(|e| ClientError::Api(Box::new(e)))?
                    .data
                    .into_iter()
                    .enumerate()
                    .map(|(id, data)| {
                        Character::new(
                            id,
                            Arc::new(RwLock::new(Arc::new(data))),
                            bank_clone.clone(),
                            items_clone.clone(),
                            resources_clone.clone(),
                            monsters_clone.clone(),
                            maps_clone.clone(),
                            npcs_clone.clone(),
                            server_clone.clone(),
                            api_clone.clone(),
                        )
                    })
                    .map(Arc::new)
                    .collect::<Vec<_>>())
            })
            .join()
            .unwrap()
        })?;

        let account = Arc::new(Account::new(account_name, bank, characters));

        Ok(Self {
            account,
            items,
            monsters,
            resources,
            server,
            events,
            tasks,
            tasks_rewards,
            maps,
            npcs,
        })
    }
}
