use super::{
    CharacterData, HasCharacterData, inventory::Inventory, request_handler::CharacterRequestHandler,
};
use crate::{
    Gear,
    bank::Bank,
    char::error::{
        BankExpansionError, BuyNpcError, CraftError, DeleteError, DepositError, EquipError,
        FightError, GatherError, GoldDepositError, GoldWithdrawError, MoveError, RecycleError,
        RestError, SellNpcError, TaskAcceptationError, TaskCancellationError, TaskCompletionError,
        TaskTradeError, TasksCoinExchangeError, UnequipError, UseError, WithdrawError,
    },
    gear::Slot,
    items::{ItemSchemaExt, Items},
    maps::{MapSchemaExt, Maps},
    monsters::{MonsterSchemaExt, Monsters},
    npcs::Npcs,
    resources::{ResourceSchemaExt, Resources},
    server::Server,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{
    CharacterSchema, FightSchema, MapContentType, MapSchema, NpcItemTransactionSchema,
    RecyclingItemsSchema, RewardsSchema, SimpleItemSchema, SkillDataSchema, SkillInfoSchema,
    TaskSchema, TaskTradeSchema,
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Character {
    pub id: usize,
    inner: CharacterRequestHandler,
    pub inventory: Arc<Inventory>,
    bank: Arc<Bank>,
    items: Arc<Items>,
    resources: Arc<Resources>,
    monsters: Arc<Monsters>,
    maps: Arc<Maps>,
    npcs: Arc<Npcs>,
    server: Arc<Server>,
}

impl Character {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: usize,
        data: CharacterData,
        bank: Arc<Bank>,
        items: Arc<Items>,
        resources: Arc<Resources>,
        monsters: Arc<Monsters>,
        maps: Arc<Maps>,
        npcs: Arc<Npcs>,
        server: Arc<Server>,
        api: Arc<ArtifactApi>,
    ) -> Self {
        Self {
            id,
            inner: CharacterRequestHandler::new(api, data.clone(), bank.clone(), server.clone()),
            inventory: Arc::new(Inventory::new(data, items.clone())),
            bank,
            items,
            resources,
            monsters,
            maps,
            npcs,
            server,
        }
    }

    pub fn current_map(&self) -> Arc<MapSchema> {
        let (x, y) = self.position();
        self.maps.get(x, y).unwrap()
    }

    pub fn fight(&self) -> Result<FightSchema, FightError> {
        self.can_fight()?;
        Ok(self.inner.request_fight()?)
    }

    pub fn can_fight(&self) -> Result<(), FightError> {
        let Some(monster_code) = self.current_map().monster() else {
            return Err(FightError::NoMonsterOnMap);
        };
        let Some(monster) = self.monsters.get(&monster_code) else {
            return Err(FightError::NoMonsterOnMap);
        };
        //TODO: take inventory free slot into account
        if self.inventory.free_space() < monster.max_drop_quantity() {
            return Err(FightError::InsufficientInventorySpace);
        }
        Ok(())
    }

    pub fn gather(&self) -> Result<SkillDataSchema, GatherError> {
        self.can_gather()?;
        Ok(self.inner.request_gather()?)
    }

    pub fn can_gather(&self) -> Result<(), GatherError> {
        let Some(resource_code) = self.current_map().resource() else {
            return Err(GatherError::NoResourceOnMap);
        };
        let Some(resource) = self.resources.get(&resource_code) else {
            return Err(GatherError::NoResourceOnMap);
        };
        if self.skill_level(resource.skill.into()) < resource.level {
            return Err(GatherError::SkillLevelInsufficient);
        }
        //TODO: take inventory free slot into account
        if self.inventory.free_space() < resource.max_drop_quantity() {
            return Err(GatherError::InsufficientInventorySpace);
        }
        Ok(())
    }

    pub fn r#move(&self, x: i32, y: i32) -> Result<Arc<MapSchema>, MoveError> {
        self.can_move(x, y)?;
        Ok(self.inner.request_move(x, y)?)
    }

    pub fn can_move(&self, x: i32, y: i32) -> Result<(), MoveError> {
        if self.position() == (x, y) {
            return Err(MoveError::AlreadyOnMap);
        }
        if self.maps.get(x, y).is_none() {
            return Err(MoveError::MapNotFound);
        }
        Ok(())
    }

    pub fn rest(&self) -> Result<i32, RestError> {
        if self.health() < self.max_health() {
            return Ok(self.inner.request_rest()?);
        }
        Ok(0)
    }

    pub fn r#use(&self, item_code: &str, quantity: i32) -> Result<(), UseError> {
        self.can_use(item_code, quantity)?;
        Ok(self.inner.request_use_item(item_code, quantity)?)
    }

    pub fn can_use(&self, item_code: &str, quantity: i32) -> Result<(), UseError> {
        let Some(item) = self.items.get(item_code) else {
            return Err(UseError::ItemNotFound);
        };
        if !item.is_consumable() {
            return Err(UseError::ItemNotConsumable);
        }
        if self.inventory.total_of(item_code) < quantity {
            return Err(UseError::InsufficientQuantity);
        }
        if self.level() < item.level {
            return Err(UseError::InsufficientCharacterLevel);
        }
        Ok(())
    }

    pub fn craft(&self, item_code: &str, quantity: i32) -> Result<SkillInfoSchema, CraftError> {
        self.can_craft(item_code, quantity)?;
        Ok(self.inner.request_craft(item_code, quantity)?)
    }

    pub fn can_craft(&self, item_code: &str, quantity: i32) -> Result<(), CraftError> {
        let Some(item) = self.items.get(item_code) else {
            return Err(CraftError::ItemNotFound);
        };
        let Some(skill) = item.skill_to_craft() else {
            return Err(CraftError::ItemNotCraftable);
        };
        if self.skill_level(skill) < item.level {
            return Err(CraftError::InsufficientSkillLevel);
        }
        if !self.inventory.contains_mats_for(item_code, quantity) {
            return Err(CraftError::InsufficientMaterials);
        }
        // TODO: check if InssuficientInventorySpace can happen
        if !self.current_map().content_code_is(skill.as_ref()) {
            return Err(CraftError::NoWorkshopOnMap);
        }
        Ok(())
    }

    pub fn recycle(
        &self,
        item_code: &str,
        quantity: i32,
    ) -> Result<RecyclingItemsSchema, RecycleError> {
        self.can_recycle(item_code, quantity)?;
        Ok(self.inner.request_recycle(item_code, quantity)?)
    }

    pub fn can_recycle(&self, item_code: &str, quantity: i32) -> Result<(), RecycleError> {
        let Some(item) = self.items.get(item_code) else {
            return Err(RecycleError::ItemNotFound);
        };
        let Some(skill) = item.skill_to_craft() else {
            return Err(RecycleError::ItemNotRecyclable);
        };
        if skill.is_cooking() || skill.is_alchemy() {
            return Err(RecycleError::ItemNotRecyclable);
        }
        if self.skill_level(skill) < item.level {
            return Err(RecycleError::InsufficientSkillLevel);
        }
        if self.inventory.total_of(item_code) < quantity {
            return Err(RecycleError::InsufficientQuantity);
        }
        if self.inventory.free_space() + quantity < item.recycled_quantity() {
            return Err(RecycleError::InsufficientInventorySpace);
        }
        if !self.current_map().content_code_is(skill.as_ref()) {
            return Err(RecycleError::NoWorkshopOnMap);
        }
        Ok(())
    }

    pub fn delete(&self, item_code: &str, quantity: i32) -> Result<SimpleItemSchema, DeleteError> {
        self.can_delete(item_code, quantity)?;
        Ok(self.inner.request_delete(item_code, quantity)?)
    }

    pub fn can_delete(&self, item_code: &str, quantity: i32) -> Result<(), DeleteError> {
        if self.items.get(item_code).is_none() {
            return Err(DeleteError::ItemNotFound);
        };
        if self.inventory.total_of(item_code) < quantity {
            return Err(DeleteError::InsufficientQuantity);
        }
        Ok(())
    }

    pub fn withdraw_item(&self, items: &[SimpleItemSchema]) -> Result<(), WithdrawError> {
        self.can_withdraw_items(items)?;
        Ok(self.inner.request_withdraw_item(items)?)
    }

    pub fn can_withdraw_items(&self, items: &[SimpleItemSchema]) -> Result<(), WithdrawError> {
        if items
            .iter()
            .any(|i| self.bank.total_of(&i.code) < i.quantity)
        {
            return Err(WithdrawError::InsufficientQuantity);
        };
        let total_quantity = items.iter().map(|i| i.quantity).sum();
        let items_already_in_inventory = items
            .iter()
            .filter(|i| self.inventory.total_of(&i.code) > 0)
            .count();
        let new_slot_taken = items.len() - items_already_in_inventory;
        if self.inventory.free_space() < total_quantity
            || self.inventory.free_slots() < new_slot_taken
        {
            return Err(WithdrawError::InsufficientInventorySpace);
        }
        if !self.current_map().content_type_is(MapContentType::Bank) {
            return Err(WithdrawError::NoBankOnMap);
        }
        Ok(())
    }

    pub fn deposit_item(&self, items: &[SimpleItemSchema]) -> Result<(), DepositError> {
        self.can_deposit_items(items)?;
        Ok(self.inner.request_deposit_item(items)?)
    }

    pub fn can_deposit_items(&self, items: &[SimpleItemSchema]) -> Result<(), DepositError> {
        //TODO: add check for bank slot availability
        for item in items.iter() {
            self.can_deposit_item(&item.code, item.quantity)?
        }
        Ok(())
    }

    fn can_deposit_item(&self, item_code: &str, quantity: i32) -> Result<(), DepositError> {
        if self.items.get(item_code).is_none() {
            return Err(DepositError::ItemNotFound);
        };
        if self.inventory.total_of(item_code) < quantity {
            return Err(DepositError::InsufficientQuantity);
        }
        if self.bank.total_of(item_code) <= 0 && self.bank.free_slots() <= 0 {
            return Err(DepositError::InsufficientBankSpace);
        }
        if !self.current_map().content_type_is(MapContentType::Bank) {
            return Err(DepositError::NoBankOnMap);
        }
        Ok(())
    }

    pub fn withdraw_gold(&self, quantity: i32) -> Result<i32, GoldWithdrawError> {
        self.can_withdraw_gold(quantity)?;
        Ok(self.inner.request_withdraw_gold(quantity)?)
    }

    pub fn can_withdraw_gold(&self, quantity: i32) -> Result<(), GoldWithdrawError> {
        if self.bank.gold() < quantity {
            return Err(GoldWithdrawError::InsufficientGold);
        }
        if !self.current_map().content_type_is(MapContentType::Bank) {
            return Err(GoldWithdrawError::NoBankOnMap);
        }
        Ok(())
    }

    pub fn deposit_gold(&self, quantity: i32) -> Result<i32, GoldDepositError> {
        self.can_deposit_gold(quantity)?;
        Ok(self.inner.request_deposit_gold(quantity)?)
    }

    pub fn can_deposit_gold(&self, quantity: i32) -> Result<(), GoldDepositError> {
        if self.gold() < quantity {
            return Err(GoldDepositError::InsufficientGold);
        }
        if !self.current_map().content_type_is(MapContentType::Bank) {
            return Err(GoldDepositError::NoBankOnMap);
        }
        Ok(())
    }

    pub fn expand_bank(&self) -> Result<i32, BankExpansionError> {
        self.can_expand_bank()?;
        Ok(self.inner.request_expand_bank()?)
    }

    pub fn can_expand_bank(&self) -> Result<(), BankExpansionError> {
        if self.gold() < self.bank.details().next_expansion_cost {
            return Err(BankExpansionError::InsufficientGold);
        }
        if !self.current_map().content_type_is(MapContentType::Bank) {
            return Err(BankExpansionError::NoBankOnMap);
        }
        Ok(())
    }

    pub fn equip(&self, item_code: &str, slot: Slot, quantity: i32) -> Result<(), EquipError> {
        self.can_equip(item_code, slot, quantity)?;
        Ok(self.inner.request_equip(item_code, slot, quantity)?)
    }

    pub fn can_equip(&self, item_code: &str, slot: Slot, quantity: i32) -> Result<(), EquipError> {
        let Some(item) = self.items.get(item_code) else {
            return Err(EquipError::ItemNotFound);
        };
        if self.inventory.total_of(item_code) < quantity {
            return Err(EquipError::InsufficientQuantity);
        }
        if let Some(equiped) = self.items.get(&self.equiped_in(slot)) {
            if equiped.code == item_code {
                if slot.max_quantity() <= 1 {
                    return Err(EquipError::ItemAlreadyEquiped);
                } else if self.quantity_in_slot(slot) + quantity > slot.max_quantity() {
                    return Err(EquipError::QuantityGreaterThanSlotMaxixum);
                }
            } else {
                return Err(EquipError::SlotNotEmpty);
            }
        }
        if !self.meets_conditions_for(&item) {
            return Err(EquipError::ConditionsNotMet);
        }
        if self.inventory.free_space() + item.inventory_space() <= 0 {
            return Err(EquipError::InsufficientInventorySpace);
        }
        Ok(())
    }

    pub fn unequip(&self, slot: Slot, quantity: i32) -> Result<(), UnequipError> {
        self.can_unequip(slot, quantity)?;
        Ok(self.inner.request_unequip(slot, quantity)?)
    }

    pub fn can_unequip(&self, slot: Slot, quantity: i32) -> Result<(), UnequipError> {
        let Some(equiped) = self.items.get(&self.equiped_in(slot)) else {
            return Err(UnequipError::SlotEmpty);
        };
        if self.health() <= equiped.health() {
            return Err(UnequipError::InsufficientHealth);
        }
        if self.quantity_in_slot(slot) < quantity {
            return Err(UnequipError::InsufficientQuantity);
        }
        if !self.inventory.has_space_for(&equiped.code, quantity) {
            return Err(UnequipError::InsufficientInventorySpace);
        }
        Ok(())
    }

    pub fn accept_task(&self) -> Result<TaskSchema, TaskAcceptationError> {
        self.can_accept_task()?;
        Ok(self.inner.request_accept_task()?)
    }

    pub fn can_accept_task(&self) -> Result<(), TaskAcceptationError> {
        if !self.task().is_empty() {
            return Err(TaskAcceptationError::TaskAlreadyInProgress);
        }
        if !self
            .current_map()
            .content_type_is(MapContentType::TasksMaster)
        {
            return Err(TaskAcceptationError::NoTasksMasterOnMap);
        }
        Ok(())
    }

    pub fn task_trade(
        &self,
        item_code: &str,
        quantity: i32,
    ) -> Result<TaskTradeSchema, TaskTradeError> {
        self.can_task_trade(item_code, quantity)?;
        Ok(self.inner.request_task_trade(item_code, quantity)?)
    }

    pub fn can_task_trade(&self, item_code: &str, quantity: i32) -> Result<(), TaskTradeError> {
        if self.items.get(item_code).is_none() {
            return Err(TaskTradeError::ItemNotFound);
        };
        if item_code != self.task() {
            return Err(TaskTradeError::WrongTask);
        }
        if self.inventory.total_of(item_code) < quantity {
            return Err(TaskTradeError::InsufficientQuantity);
        }
        if self.task_missing() < quantity {
            return Err(TaskTradeError::SuperfluousQuantity);
        }
        if !self
            .current_map()
            .content_type_is(MapContentType::TasksMaster)
            || !self.current_map().content_code_is("items")
        {
            return Err(TaskTradeError::WrongOrNoTasksMasterOnMap);
        }
        Ok(())
    }

    pub fn complete_task(&self) -> Result<RewardsSchema, TaskCompletionError> {
        self.can_complete_task()?;
        Ok(self.inner.request_complete_task()?)
    }

    pub fn can_complete_task(&self) -> Result<(), TaskCompletionError> {
        let Some(task_type) = self.task_type() else {
            return Err(TaskCompletionError::NoCurrentTask);
        };
        if !self.task_finished() {
            return Err(TaskCompletionError::TaskNotFullfilled);
        }
        if self.inventory.free_space() < 2 {
            return Err(TaskCompletionError::InsufficientInventorySpace);
        }
        if !self
            .current_map()
            .content_type_is(MapContentType::TasksMaster)
            || !self.current_map().content_code_is(&task_type.to_string())
        {
            return Err(TaskCompletionError::WrongOrNoTasksMasterOnMap);
        }
        Ok(())
    }

    pub fn cancel_task(&self) -> Result<(), TaskCancellationError> {
        self.can_cancel_task()?;
        Ok(self.inner.request_cancel_task()?)
    }

    pub fn can_cancel_task(&self) -> Result<(), TaskCancellationError> {
        let Some(task_type) = self.task_type() else {
            return Err(TaskCancellationError::NoCurrentTask);
        };
        if self.inventory.total_of("tasks_coin") < 1 {
            return Err(TaskCancellationError::InsufficientTasksCoinQuantity);
        }
        if !self
            .current_map()
            .content_type_is(MapContentType::TasksMaster)
            || !self.current_map().content_code_is(&task_type.to_string())
        {
            return Err(TaskCancellationError::WrongOrNoTasksMasterOnMap);
        }
        Ok(())
    }

    pub fn exchange_tasks_coin(&self) -> Result<RewardsSchema, TasksCoinExchangeError> {
        self.can_exchange_tasks_coin()?;
        Ok(self.inner.request_task_exchange()?)
    }

    pub fn can_exchange_tasks_coin(&self) -> Result<(), TasksCoinExchangeError> {
        if self.inventory.total_of("tasks_coin") < 6 {
            return Err(TasksCoinExchangeError::InsufficientTasksCoinQuantity);
        }
        if !self
            .current_map()
            .content_type_is(MapContentType::TasksMaster)
        {
            return Err(TasksCoinExchangeError::NoTasksMasterOnMap);
        }
        // TODO: check for conditions when InsufficientInventorySpace can happen
        Ok(())
    }

    pub fn npc_buy(
        &self,
        item_code: &str,
        quantity: i32,
    ) -> Result<NpcItemTransactionSchema, BuyNpcError> {
        self.can_npc_buy(item_code, quantity)?;
        Ok(self.inner.request_npc_buy(item_code, quantity)?)
    }

    fn can_npc_buy(&self, item_code: &str, quantity: i32) -> Result<(), BuyNpcError> {
        if self.items.get(item_code).is_none() {
            return Err(BuyNpcError::ItemNotFound);
        };
        let Some(item) = self.npcs.items.get(item_code) else {
            return Err(BuyNpcError::ItemNotBuyable);
        };
        let Some(buy_price) = item.buy_price else {
            return Err(BuyNpcError::ItemNotBuyable);
        };
        if item.currency == "gold" {
            if self.gold() < buy_price * quantity {
                return Err(BuyNpcError::InsufficientGold);
            }
        } else if self.inventory.total_of(&item.currency) < buy_price * quantity {
            return Err(BuyNpcError::InsufficientQuantity);
        }
        Ok(())
    }

    pub fn npc_sell(
        &self,
        item_code: &str,
        quantity: i32,
    ) -> Result<NpcItemTransactionSchema, SellNpcError> {
        self.can_npc_sell(item_code, quantity)?;
        Ok(self.inner.request_npc_sell(item_code, quantity)?)
    }

    fn can_npc_sell(&self, item_code: &str, quantity: i32) -> Result<(), SellNpcError> {
        if self.items.get(item_code).is_none() {
            return Err(SellNpcError::ItemNotFound);
        };
        let Some(item) = self.npcs.items.get(item_code) else {
            return Err(SellNpcError::ItemNotSellable);
        };
        if item.sell_price.is_none() {
            return Err(SellNpcError::ItemNotSellable);
        };
        if self.inventory.total_of(item_code) < quantity {
            return Err(SellNpcError::InsufficientQuantity);
        }
        Ok(())
    }

    //pub fn exchange_gift(&self) -> Result<RewardsSchema, GiftExchangeError> {
    //    self.can_exchange_gift()?;
    //    Ok(self.inner.request_gift_exchange()?)
    //}

    // pub fn can_exchange_gift(&self) -> Result<(), GiftExchangeError> {
    //     if self.inventory.total_of("tasks_coin") < 1 {
    //         return Err(GiftExchangeError::InsufficientGiftQuantity);
    //     }
    //     if !self.map().content_type_is(MapContentType::SantaClaus) {
    //         return Err(GiftExchangeError::NoSantaClausOnMap);
    //     }
    //     // TODO: check for conditions when InsufficientInventorySpace can happen
    //     Ok(())
    // }

    pub fn gear(&self) -> Gear {
        let d = self.data();
        Gear {
            weapon: self.items.get(&d.weapon_slot),
            shield: self.items.get(&d.shield_slot),
            helmet: self.items.get(&d.helmet_slot),
            body_armor: self.items.get(&d.body_armor_slot),
            leg_armor: self.items.get(&d.leg_armor_slot),
            boots: self.items.get(&d.boots_slot),
            ring1: self.items.get(&d.ring1_slot),
            ring2: self.items.get(&d.ring2_slot),
            amulet: self.items.get(&d.amulet_slot),
            artifact1: self.items.get(&d.artifact1_slot),
            artifact2: self.items.get(&d.artifact2_slot),
            artifact3: self.items.get(&d.artifact3_slot),
            utility1: self.items.get(&d.utility1_slot),
            utility2: self.items.get(&d.utility2_slot),
        }
    }
}

impl HasCharacterData for Character {
    fn data(&self) -> Arc<CharacterSchema> {
        self.inner.data()
    }

    fn server(&self) -> Arc<Server> {
        self.server.clone()
    }

    fn refresh_data(&self) {
        self.inner.refresh_data();
    }

    fn update_data(&self, schema: CharacterSchema) {
        self.inner.update_data(schema);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifactsmmo_openapi::models::InventorySlot;
    use std::sync::RwLock;

    impl From<CharacterSchema> for Character {
        fn from(value: CharacterSchema) -> Self {
            Self::new(
                1,
                Arc::new(RwLock::new(Arc::new(value))),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            )
        }
    }

    //TODO: fix test
    // #[test]
    // fn can_fight() {
    //     // monster on 0,2 is "cow"
    //     let char = BaseCharacter::from(CharacterSchema {
    //         x: 0,
    //         y: 2,
    //         inventory_max_items: 100,
    //         ..Default::default()
    //     });
    //     assert!(char.can_fight().is_ok());
    //     let char = BaseCharacter::from(CharacterSchema {
    //         x: 0,
    //         y: 2,
    //         inventory_max_items: &char.map().monster().unwrap().max_drop_quantity() - 1,
    //         ..Default::default()
    //     });
    //     assert!(matches!(
    //         char.can_fight(),
    //         Err(FightError::InsufficientInventorySpace)
    //     ));
    // }

    //TODO: fix test
    // #[test]
    // fn can_gather() {
    //     let char = BaseCharacter::from(CharacterSchema {
    //         x: 2,
    //         y: 0,
    //         mining_level: 1,
    //         inventory_max_items: 100,
    //         ..Default::default()
    //     });
    //     assert!(char.can_gather().is_ok());
    //     let char = BaseCharacter::from(CharacterSchema {
    //         x: 0,
    //         y: 0,
    //         mining_level: 1,
    //         ..Default::default()
    //     });
    //     assert!(matches!(
    //         char.can_gather(),
    //         Err(GatherError::NoResourceOnMap)
    //     ));
    //     let char = BaseCharacter::from(CharacterSchema {
    //         x: 1,
    //         y: 7,
    //         mining_level: 1,
    //         ..Default::default()
    //     });
    //     assert!(matches!(
    //         char.can_gather(),
    //         Err(GatherError::SkillLevelInsufficient)
    //     ));
    //     let char = BaseCharacter::from(CharacterSchema {
    //         x: 2,
    //         y: 0,
    //         mining_level: 1,
    //         inventory_max_items: char
    //             .maps
    //             .get(2, 0)
    //             .unwrap()
    //             .resource()
    //             .unwrap()
    //             .max_drop_quantity()
    //             - 1,
    //         ..Default::default()
    //     });
    //     assert!(matches!(
    //         char.can_gather(),
    //         Err(GatherError::InsufficientInventorySpace)
    //     ));
    // }

    #[test]
    fn can_move() {
        let char = Character::from(CharacterSchema::default());
        assert!(char.can_move(0, 0).is_ok());
        assert!(matches!(
            char.can_move(1000, 0),
            Err(MoveError::MapNotFound)
        ));
    }

    #[test]
    fn can_use() {
        let item1 = "cooked_chicken";
        let item2 = "cooked_shrimp";
        let char = Character::from(CharacterSchema {
            level: 5,
            inventory: Some(vec![
                InventorySlot {
                    slot: 0,
                    code: item1.to_owned(),
                    quantity: 1,
                },
                InventorySlot {
                    slot: 1,
                    code: item2.to_owned(),
                    quantity: 1,
                },
            ]),
            ..Default::default()
        });
        assert!(matches!(
            char.can_use("random_item", 1),
            Err(UseError::ItemNotFound)
        ));
        assert!(matches!(
            char.can_use("copper", 1),
            Err(UseError::ItemNotConsumable)
        ));
        assert!(matches!(
            char.can_use(item1, 5),
            Err(UseError::InsufficientQuantity)
        ));
        assert!(matches!(
            char.can_use(item2, 1),
            Err(UseError::InsufficientCharacterLevel)
        ));
        assert!(char.can_use(item1, 1).is_ok());
    }

    #[test]
    fn can_craft() {
        let char = Character::from(CharacterSchema {
            cooking_level: 1,
            inventory: Some(vec![
                InventorySlot {
                    slot: 0,
                    code: "gudgeon".to_string(),
                    quantity: 1,
                },
                InventorySlot {
                    slot: 1,
                    code: "shrimp".to_string(),
                    quantity: 1,
                },
            ]),
            inventory_max_items: 100,
            ..Default::default()
        });
        assert!(matches!(
            char.can_craft("random_item", 1),
            Err(CraftError::ItemNotFound)
        ));
        assert!(matches!(
            char.can_craft("copper_ore", 1),
            Err(CraftError::ItemNotCraftable)
        ));
        assert!(matches!(
            char.can_craft("cooked_chicken", 1),
            Err(CraftError::InsufficientMaterials)
        ));
        assert!(matches!(
            char.can_craft("cooked_gudgeon", 5),
            Err(CraftError::InsufficientMaterials)
        ));
        assert!(matches!(
            char.can_craft("cooked_shrimp", 1),
            Err(CraftError::InsufficientSkillLevel)
        ));
        assert!(matches!(
            char.can_craft("cooked_gudgeon", 1),
            Err(CraftError::NoWorkshopOnMap)
        ));
        let char = Character::from(CharacterSchema {
            cooking_level: 1,
            inventory: Some(vec![InventorySlot {
                slot: 0,
                code: "gudgeon".to_string(),
                quantity: 1,
            }]),
            inventory_max_items: 100,
            x: 1,
            y: 1,
            ..Default::default()
        });
        assert!(char.can_craft("cooked_gudgeon", 1).is_ok());
    }

    #[test]
    fn can_recycle() {
        let char = Character::from(CharacterSchema {
            cooking_level: 1,
            weaponcrafting_level: 1,
            inventory: Some(vec![
                InventorySlot {
                    slot: 0,
                    code: "copper_dagger".to_string(),
                    quantity: 1,
                },
                InventorySlot {
                    slot: 1,
                    code: "iron_sword".to_string(),
                    quantity: 1,
                },
                InventorySlot {
                    slot: 2,
                    code: "cooked_gudgeon".to_string(),
                    quantity: 1,
                },
            ]),
            inventory_max_items: 100,
            ..Default::default()
        });
        assert!(matches!(
            char.can_recycle("random_item", 1),
            Err(RecycleError::ItemNotFound)
        ));
        assert!(matches!(
            char.can_recycle("cooked_gudgeon", 1),
            Err(RecycleError::ItemNotRecyclable)
        ));
        assert!(matches!(
            char.can_recycle("wooden_staff", 1),
            Err(RecycleError::InsufficientQuantity)
        ));
        assert!(matches!(
            char.can_recycle("iron_sword", 1),
            Err(RecycleError::InsufficientSkillLevel)
        ));
        assert!(matches!(
            char.can_recycle("copper_dagger", 1),
            Err(RecycleError::NoWorkshopOnMap)
        ));
        let char = Character::from(CharacterSchema {
            weaponcrafting_level: 1,
            inventory: Some(vec![InventorySlot {
                slot: 0,
                code: "copper_dagger".to_string(),
                quantity: 1,
            }]),
            inventory_max_items: 1,
            x: 2,
            y: 1,
            ..Default::default()
        });
        assert!(matches!(
            char.can_recycle("copper_dagger", 1),
            Err(RecycleError::InsufficientInventorySpace)
        ));
        let char = Character::from(CharacterSchema {
            weaponcrafting_level: 1,
            inventory: Some(vec![InventorySlot {
                slot: 0,
                code: "copper_dagger".to_string(),
                quantity: 1,
            }]),
            inventory_max_items: 100,
            x: 2,
            y: 1,
            ..Default::default()
        });
        assert!(char.can_recycle("copper_dagger", 1).is_ok());
    }

    #[test]
    fn can_delete() {
        let char = Character::from(CharacterSchema {
            cooking_level: 1,
            weaponcrafting_level: 1,
            inventory: Some(vec![InventorySlot {
                slot: 0,
                code: "copper_dagger".to_string(),
                quantity: 1,
            }]),
            inventory_max_items: 100,
            ..Default::default()
        });
        assert!(matches!(
            char.can_delete("random_item", 1),
            Err(DeleteError::ItemNotFound)
        ));
        assert!(matches!(
            char.can_delete("copper_dagger", 2),
            Err(DeleteError::InsufficientQuantity)
        ));
        assert!(char.can_delete("copper_dagger", 1).is_ok());
    }

    #[test]
    fn can_withdraw() {
        let char = Character::from(CharacterSchema {
            inventory_max_items: 100,
            ..Default::default()
        });
        char.bank.update_content(vec![
            SimpleItemSchema {
                code: "copper_dagger".to_string(),
                quantity: 1,
            },
            SimpleItemSchema {
                code: "iron_sword".to_string(),
                quantity: 101,
            },
        ]);
        // TODO: rewrite these tests
        // assert!(matches!(
        //     char.can_withdraw_items("random_item", 1),
        //     Err(WithdrawError::ItemNotFound)
        // ));
        // assert!(matches!(
        //     char.can_withdraw_item("copper_dagger", 2),
        //     Err(WithdrawError::InsufficientQuantity)
        // ));
        // assert!(matches!(
        //     char.can_withdraw_item("iron_sword", 101),
        //     Err(WithdrawError::InsufficientInventorySpace)
        // ));
        // assert!(matches!(
        //     char.can_withdraw_item("iron_sword", 10),
        //     Err(WithdrawError::NoBankOnMap)
        // ));
        let char = Character::from(CharacterSchema {
            inventory_max_items: 100,
            x: 4,
            y: 1,
            ..Default::default()
        });
        char.bank.update_content(vec![
            SimpleItemSchema {
                code: "copper_dagger".to_string(),
                quantity: 1,
            },
            SimpleItemSchema {
                code: "iron_sword".to_string(),
                quantity: 101,
            },
        ]);
        // assert!(char.can_withdraw_item("iron_sword", 10).is_ok());
    }
    //TODO: add more tests
}
