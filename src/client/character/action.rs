use crate::{client::character::error::RequestError, gear::Slot};

use super::request_handler::ResponseSchema;
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::SimpleItemSchema;
use strum_macros::{Display, EnumIs};

#[derive(Debug, EnumIs, Display)]
pub enum Action<'a> {
    Move {
        x: i32,
        y: i32,
    },
    Fight,
    Rest,
    UseItem {
        item: &'a str,
        quantity: u32,
    },
    Gather,
    Craft {
        item: &'a str,
        quantity: u32,
    },
    Recycle {
        item: &'a str,
        quantity: u32,
    },
    Delete {
        item: &'a str,
        quantity: u32,
    },
    DepositItem {
        items: &'a [SimpleItemSchema],
    },
    WithdrawItem {
        items: &'a [SimpleItemSchema],
    },
    DepositGold {
        quantity: u32,
    },
    WithdrawGold {
        quantity: u32,
    },
    ExpandBank,
    Equip {
        item: &'a str,
        slot: Slot,
        quantity: u32,
    },
    Unequip {
        slot: Slot,
        quantity: u32,
    },
    AcceptTask,
    TaskTrade {
        item: &'a str,
        quantity: u32,
    },
    CompleteTask,
    CancelTask,
    TaskExchange,
    NpcBuy {
        item: &'a str,
        quantity: u32,
    },
    NpcSell {
        item: &'a str,
        quantity: u32,
    },
    GiveItem {
        items: &'a [SimpleItemSchema],
        character: &'a str,
    },
    GiveGold {
        quantity: u32,
        character: &'a str,
    },
    //ChristmasExchange,
}

impl Action<'_> {
    pub fn request(
        &self,
        name: &str,
        api: &ArtifactApi,
    ) -> Result<Box<dyn ResponseSchema>, RequestError> {
        match self {
            Action::Move { x, y } => api
                .my_character
                .r#move(name, *x, *y)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Fight => api
                .my_character
                .fight(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Rest => api
                .my_character
                .rest(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::UseItem { item, quantity } => api
                .my_character
                .use_item(name, item, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Gather => api
                .my_character
                .gather(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Craft { item, quantity } => api
                .my_character
                .craft(name, item, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Recycle { item, quantity } => api
                .my_character
                .recycle(name, item, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Delete { item, quantity } => api
                .my_character
                .delete(name, item, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::DepositItem { items } => api
                .my_character
                .deposit(name, items)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::WithdrawItem { items } => api
                .my_character
                .withdraw(name, items)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::DepositGold { quantity } => api
                .my_character
                .deposit_gold(name, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::WithdrawGold { quantity } => api
                .my_character
                .withdraw_gold(name, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::ExpandBank => api
                .my_character
                .expand_bank(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Equip {
                item,
                slot,
                quantity,
            } => api
                .my_character
                .equip(name, item, (*slot).into(), Some(*quantity))
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::Unequip { slot, quantity } => {
                api.my_character
                    .unequip(name, (*slot).into(), Some(*quantity))
            }
            .map(|r| r.into())
            .map_err(|e| e.into()),
            Action::AcceptTask => api
                .my_character
                .accept_task(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::TaskTrade { item, quantity } => api
                .my_character
                .trade_task(name, item, *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::CompleteTask => api
                .my_character
                .complete_task(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::CancelTask => api
                .my_character
                .cancel_task(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::TaskExchange => api
                .my_character
                .task_exchange(name)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::NpcBuy { item, quantity } => api
                .my_character
                .npc_buy(name, item.to_string(), *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::NpcSell { item, quantity } => api
                .my_character
                .npc_sell(name, item.to_string(), *quantity)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::GiveItem { items, character } => api
                .my_character
                .give_item(name, items, character)
                .map(|r| r.into())
                .map_err(|e| e.into()),
            Action::GiveGold {
                quantity,
                character,
            } => api
                .my_character
                .give_gold(name, *quantity, character)
                .map(|r| r.into())
                .map_err(|e| e.into()),
        }
    }
}
