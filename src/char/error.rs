use crate::char::request_handler::RequestError;
use derive_more::TryFrom;
use thiserror::Error;

const ENTITY_NOT_FOUND: isize = 404;
const ITEM_NOT_BUYABLE: isize = 441;
const ITEM_NOT_SELLABLE: isize = 442;
const BANK_GOLD_INSUFFICIENT: isize = 460;
//const TRANSACTION_ALREADY_IN_PROGRESS: isize = 461;
const BANK_FULL: isize = 462;
const ITEM_NOT_RECYCLABLE: isize = 473;
const WRONG_TASK: isize = 474;
const TASK_ALREADY_COMPLETED_OR_TOO_MANY_ITEM_TRADED: isize = 475;
const ITEM_NOT_CONSUMABLE: isize = 476;
const MISSING_ITEM_OR_INSUFFICIENT_QUANTITY: isize = 478;
const INSUFFICIENT_HEALTH: isize = 483;
const SUPERFLOUS_UTILITY_QUANTITY: isize = 484;
const ITEM_ALREADY_EQUIPED: isize = 485;
//const ACTION_ALREADY_IN_PROGRESS: isize = 486;
const NO_TASK: isize = 487;
const TASK_NOT_COMPLETED: isize = 488;
const TASK_ALREADY_IN_PROGRESS: isize = 489;
const ALREADY_ON_MAP: isize = 490;
const INVALID_SLOT_STATE: isize = 491;
const CHARACTER_GOLD_INSUFFICIENT: isize = 492;
const SKILL_LEVEL_INSUFFICIENT: isize = 493;
const CHARACTER_LEVEL_INSUFFICIENT: isize = 496;
const INVENTORY_FULL: isize = 497;
//const CHARACTER_NOT_FOUND: isize = 498;
//const CHARACTER_ON_COOLDOWN: isize = 499;
const ENTITY_NOT_FOUND_ON_MAP: isize = 598;

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum FightError {
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("No monster on map")]
    NoMonsterOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum GatherError {
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("No resource on map")]
    NoResourceOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error("Insufficient skill level")]
    SkillLevelInsufficient = SKILL_LEVEL_INSUFFICIENT,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum MoveError {
    #[error("Map not found")]
    MapNotFound = ENTITY_NOT_FOUND,
    #[error("Already on map")]
    AlreadyOnMap = ALREADY_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum RestError {
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum UseError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("Item not equipped")]
    ItemNotConsumable = ITEM_NOT_CONSUMABLE,
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient character level")]
    InsufficientCharacterLevel = CHARACTER_LEVEL_INSUFFICIENT,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum CraftError {
    #[error("Item not found")]
    ItemNotFound,
    #[error("Item not craftable")]
    ItemNotCraftable = ENTITY_NOT_FOUND,
    #[error("Insufficient materials")]
    InsufficientMaterials = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient skill level")]
    InsufficientSkillLevel = SKILL_LEVEL_INSUFFICIENT,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("Required workshop not on map")]
    NoWorkshopOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum RecycleError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("Item not recyclable")]
    ItemNotRecyclable = ITEM_NOT_RECYCLABLE,
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("Insufficient skill level")]
    InsufficientSkillLevel = SKILL_LEVEL_INSUFFICIENT,
    #[error("Required workshop not on map")]
    NoWorkshopOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum DeleteError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum WithdrawError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("No bank on map")]
    NoBankOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum DepositError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient bank space")]
    InsufficientBankSpace = BANK_FULL,
    #[error("No bank on map")]
    NoBankOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum GoldWithdrawError {
    #[error("Insufficient gold in bank")]
    InsufficientGold = BANK_GOLD_INSUFFICIENT,
    #[error("No bank on map")]
    NoBankOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum GoldDepositError {
    #[error("Insufficient gold on character")]
    InsufficientGold = CHARACTER_GOLD_INSUFFICIENT,
    #[error("No bank on map")]
    NoBankOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum BankExpansionError {
    #[error("Insufficient gold on character")]
    InsufficientGold = CHARACTER_GOLD_INSUFFICIENT,
    #[error("No bank on map")]
    NoBankOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum EquipError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Item already equiped")]
    ItemAlreadyEquiped = ITEM_ALREADY_EQUIPED,
    #[error("Quantity greater than slot max quantity")]
    QuantityGreaterThanSlotMaxixum = SUPERFLOUS_UTILITY_QUANTITY,
    #[error("Slot not empty")]
    SlotNotEmpty = INVALID_SLOT_STATE,
    #[error("Conditions not met")]
    ConditionsNotMet = CHARACTER_LEVEL_INSUFFICIENT,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum UnequipError {
    #[error("Insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient health")]
    InsufficientHealth = INSUFFICIENT_HEALTH,
    #[error("Slot is empty")]
    SlotEmpty = INVALID_SLOT_STATE,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum TaskAcceptationError {
    #[error("Task already in progress")]
    TaskAlreadyInProgress = TASK_ALREADY_IN_PROGRESS,
    #[error("No tasks master on map")]
    NoTasksMasterOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum TaskTradeError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("WrongTask")]
    WrongTask = WRONG_TASK,
    #[error("Superfluous quantity")]
    SuperfluousQuantity = TASK_ALREADY_COMPLETED_OR_TOO_MANY_ITEM_TRADED,
    #[error("InsufficientQuantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Wrong or no tasks master on map")]
    WrongOrNoTasksMasterOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum TaskCompletionError {
    #[error("No current task")]
    NoCurrentTask = NO_TASK,
    #[error("Task not fullfilled")]
    TaskNotFullfilled = TASK_NOT_COMPLETED,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("Wrong or no tasks master on map")]
    WrongOrNoTasksMasterOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum TaskCancellationError {
    #[error("No current task")]
    NoCurrentTask = NO_TASK,
    #[error("Insufficient tasks coin quantity")]
    InsufficientTasksCoinQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Wrong or no tasks master on map")]
    WrongOrNoTasksMasterOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum TasksCoinExchangeError {
    #[error("Insufficient tasks coin quantity")]
    InsufficientTasksCoinQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("No tasks master on map")]
    NoTasksMasterOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum BuyNpcError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("This item cannot be bought")]
    ItemNotBuyable = ITEM_NOT_BUYABLE,
    #[error("Insufficient gold on character")]
    InsufficientGold = CHARACTER_GOLD_INSUFFICIENT,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("Npc not found on map")]
    NpcNotFound = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum SellNpcError {
    #[error("Item not found")]
    ItemNotFound = ENTITY_NOT_FOUND,
    #[error("This item cannot be sold")]
    ItemNotSellable = ITEM_NOT_SELLABLE,
    #[error("Missing item or insufficient quantity")]
    InsufficientQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("Npc not found on map")]
    NpcNotFound = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}

#[derive(Debug, Error, TryFrom)]
#[try_from(repr)]
#[repr(isize)]
pub enum GiftExchangeError {
    #[error("Insufficient gift quantity")]
    InsufficientGiftQuantity = MISSING_ITEM_OR_INSUFFICIENT_QUANTITY,
    #[error("Insufficient inventory space")]
    InsufficientInventorySpace = INVENTORY_FULL,
    #[error("No Santa Claus on map")]
    NoSantaClausOnMap = ENTITY_NOT_FOUND_ON_MAP,
    #[error(transparent)]
    UnhandledError(#[from] RequestError),
}
