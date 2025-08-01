use crate::{
    char::Skill,
    consts::{
        ASTRALYTE_CRYSTAL, DIAMOND, ENCHANTED_FABRIC, FOOD_BLACK_LIST, JASPER_CRYSTAL,
        MAGICAL_CURE, TASKS_COIN,
    },
    gear::Slot,
    monsters::{MonsterSchemaExt, Monsters},
    resources::{ResourceSchemaExt, Resources},
    tasks_rewards::TasksRewards,
    PersistedData, Simulator,
};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{
    CraftSchema, ItemSchema, MonsterSchema, ResourceSchema, SimpleEffectSchema, SimpleItemSchema,
};
use futures::{stream, StreamExt};
use itertools::Itertools;
use log::debug;
use std::{collections::HashMap, fmt, str::FromStr, sync::Arc, vec::Vec};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIs, EnumIter, EnumString};
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Items {
    data: RwLock<HashMap<String, Arc<ItemSchema>>>,
    api: Arc<ArtifactApi>,
    resources: Arc<Resources>,
    monsters: Arc<Monsters>,
    tasks_rewards: Arc<TasksRewards>,
}

impl PersistedData<HashMap<String, Arc<ItemSchema>>> for Items {
    const PATH: &'static str = ".cache/items.json";

    async fn data_from_api(&self) -> HashMap<String, Arc<ItemSchema>> {
        self.api
            .items
            .all()
            .await
            .unwrap()
            .into_iter()
            .map(|item| (item.code.clone(), Arc::new(item)))
            .collect()
    }

    async fn refresh_data(&self) {
        *self.data.write().await = self.data_from_api().await;
    }
}

impl Items {
    pub(crate) async fn new(
        api: Arc<ArtifactApi>,
        resources: Arc<Resources>,
        monsters: Arc<Monsters>,
        tasks_rewards: Arc<TasksRewards>,
    ) -> Self {
        let items = Self {
            data: Default::default(),
            api,
            resources,
            monsters,
            tasks_rewards,
        };
        *items.data.write().await = items.retrieve_data().await;
        items
    }

    /// Takes an item `code` and return its schema.
    pub async fn get(&self, code: &str) -> Option<Arc<ItemSchema>> {
        self.data.read().await.get(code).cloned()
    }

    pub async fn all(&self) -> Vec<Arc<ItemSchema>> {
        self.data.read().await.values().cloned().collect_vec()
    }

    /// Takes an item `code` and return the mats required to craft it.
    pub async fn mats_of(&self, code: &str) -> Vec<SimpleItemSchema> {
        self.get(code)
            .await
            .iter()
            .flat_map(|i| i.mats())
            .collect_vec()
    }

    /// Takes an item `code` and returns the mats down to the raw materials
    /// required to craft it.
    pub async fn base_mats_of(&self, code: &str) -> Vec<SimpleItemSchema> {
        stream::iter(self.mats_of(code).await.iter())
            .flat_map(async |mat| {
                if self.mats_of(&mat.code).await.is_empty() {
                    vec![SimpleItemSchema {
                        code: mat.code.clone(),
                        quantity: mat.quantity,
                    }]
                } else {
                    self.mats_of(&mat.code)
                        .await
                        .iter()
                        .map(|b| SimpleItemSchema {
                            code: b.code.clone(),
                            quantity: b.quantity * mat.quantity,
                        })
                        .collect_vec()
                }
            })
            .collect::<Vec<_>>()
            .await
    }

    /// Takes an `resource` code and returns the items that can be crafted
    /// from the base mats it drops.
    pub fn crafted_from_resource(&self, resource: &str) -> Vec<Arc<ItemSchema>> {
        self.resources
            .get(resource)
            .iter()
            .flat_map(|r| {
                r.drops
                    .iter()
                    .map(|drop| self.crafted_with_base_mat(&drop.code))
                    .collect_vec()
            })
            .flatten()
            .collect_vec()
    }

    /// Takes an item `code` and returns the items directly crafted with it.
    pub async fn crafted_with(&self, code: &str) -> Vec<Arc<ItemSchema>> {
        self.all()
            .await
            .iter()
            .filter(|i| i.is_crafted_with(code))
            .cloned()
            .collect_vec()
    }

    pub async fn require_task_reward(&self, code: &str) -> bool {
        self.base_mats_of(code).await.iter().any(|m| {
            [
                JASPER_CRYSTAL,
                MAGICAL_CURE,
                ENCHANTED_FABRIC,
                ASTRALYTE_CRYSTAL,
            ]
            .contains(&m.code.as_str())
        })
    }

    /// Takes an item `code` and returns the only item it can be crafted in, or
    /// `None` otherwise.
    pub async fn unique_craft(&self, code: &str) -> Option<Arc<ItemSchema>> {
        let crafts = self.crafted_with(code).await;
        if crafts.len() == 1 {
            return Some(crafts[0].clone());
        }
        None
    }

    /// Takes an item `code` and returns the items crafted with it as base mat.
    pub async fn crafted_with_base_mat(&self, code: &str) -> Vec<Arc<ItemSchema>> {
        stream::iter(self.all().await)
            .filter_map(|i| async {
                if self.is_crafted_with_base_mat(&i.code, code).await {
                    Some(i)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .await
    }

    /// Takes an item `code` and checks if it is crafted with `mat` as a base
    /// material.
    pub async fn is_crafted_with_base_mat(&self, code: &str, mat: &str) -> bool {
        self.base_mats_of(code).await.iter().any(|m| m.code == mat)
    }

    pub async fn mats_mob_average_lvl(&self, code: &str) -> i32 {
        let mob_mats = stream::iter(self.mats_of(code).await.iter())
            .filter_map(async |i| {
                self.get(&i.code).await.and_then(|i| {
                    if i.subtype == SubType::Mob {
                        Some(i)
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>()
            .await;
        let len = mob_mats.len();
        if len > 0 {
            return mob_mats.iter().map(|i| i.level).sum::<i32>() / mob_mats.len() as i32;
        }
        0
    }

    pub async fn mats_mob_max_lvl(&self, code: &str) -> i32 {
        self.mats_of(code)
            .await
            .iter()
            .filter_map(async |i| self.get(&i.code).await)
            .filter(|i| i.subtype == SubType::Mob)
            .max_by_key(|i| i.level)
            .map_or(0, |i| i.level)
    }

    /// Takes an item `code` and returns the amount of inventory space the mats
    /// required to craft it are taking.
    pub async fn mats_quantity_for(&self, code: &str) -> i32 {
        self.mats_of(code)
            .await
            .iter()
            .map(|mat| mat.quantity)
            .sum()
    }

    pub async fn recycled_quantity_for(&self, code: &str) -> i32 {
        let mats_quantity_for = self.mats_quantity_for(code).await;
        mats_quantity_for / 5 + if mats_quantity_for % 5 > 0 { 1 } else { 0 }
    }

    /// Takes an item `code` and returns the best (lowest value) drop rate from
    /// `Monsters` or `Resources`
    pub fn drop_rate(&self, code: &str) -> i32 {
        self.monsters
            .dropping(code)
            .iter()
            .flat_map(|m| &m.drops)
            .chain(self.resources.dropping(code).iter().flat_map(|m| &m.drops))
            .find(|d| d.code == code)
            .map_or(0, |d| {
                (d.rate as f32 * ((d.min_quantity + d.max_quantity) as f32 / 2.0)).round() as i32
            })
    }

    /// Takes an item `code` and aggregate the drop rates of its base materials
    /// to cumpute an average drop rate.
    pub async fn base_mats_drop_rate(&self, code: &str) -> f32 {
        let base_mats = self.base_mats_of(code).await;
        if base_mats.is_empty() {
            return 0.0;
        }
        let base_mats_quantity: i32 = base_mats.iter().map(|m| m.quantity).sum();
        debug!("total mats for {}: {}", code, base_mats_quantity);
        let drop_rate_sum: i32 = base_mats
            .iter()
            .map(|m| self.drop_rate(&m.code) * m.quantity)
            .sum();
        debug!("sum for {}: {}", code, drop_rate_sum);
        let average: f32 = drop_rate_sum as f32 / base_mats_quantity as f32;
        debug!("average drop rate for {}: {}", code, average);
        average
    }

    pub async fn equipable_at_level(&self, level: i32, r#type: Type) -> Vec<Arc<ItemSchema>> {
        self.all()
            .await
            .iter()
            .filter(|i| i.r#type == r#type && i.level <= level)
            .cloned()
            .collect_vec()
    }

    pub async fn best_consumable_foods(&self, level: i32) -> Vec<Arc<ItemSchema>> {
        self.all()
            .await
            .iter()
            .filter(|i| i.is_consumable_at(level))
            .cloned()
            .collect_vec()
    }

    pub async fn restoring_utilities(&self, level: i32) -> Vec<Arc<ItemSchema>> {
        self.all()
            .await
            .iter()
            .filter(|i| i.r#type().is_utility() && i.restore() > 0 && i.level >= level)
            .cloned()
            .collect_vec()
    }

    /// NOTE: WIP: there is a lot of edge cases here:
    /// if all sources are resources or monsters, then the lowest drop rate source should be returned,
    /// if the drop rate sources is the same for all sources (algea), either the sources also
    /// containing other item ordereds should be returned, otherwise the one with the higest(lowest
    /// for speed?) level, or time to kill
    /// (or archivment maybe).
    /// All this logic should probably be done elsewhere since it can be related to the orderboard
    /// or the character level/skill_level/gear.
    pub fn best_source_of(&self, code: &str) -> Option<ItemSource> {
        if code == "gift" {
            return Some(ItemSource::Monster(
                self.monsters.get("gingerbread").unwrap(),
            ));
        }
        let sources = self.sources_of(code);
        if sources.iter().all(|s| s.is_resource() || s.is_monster()) {
            let bests = self.sources_of(code).into_iter().min_set_by_key(|s| {
                if let ItemSource::Resource(r) = s {
                    r.drop_rate(code)
                } else if let ItemSource::Monster(m) = s {
                    m.drop_rate(code)
                } else {
                    None
                }
            });
            bests.first().cloned()
        } else {
            sources.first().cloned()
        }
    }

    pub fn sources_of(&self, code: &str) -> Vec<ItemSource> {
        let mut sources = self
            .resources
            .dropping(code)
            .into_iter()
            .map(ItemSource::Resource)
            .collect_vec();
        sources.extend(
            self.monsters
                .dropping(code)
                .into_iter()
                .map(ItemSource::Monster)
                .collect_vec(),
        );
        if self.get(code).is_some_and(|i| i.craft_schema().is_some()) {
            sources.push(ItemSource::Craft);
        }
        if self.tasks_rewards.all().iter().any(|r| r.code == code) {
            sources.push(ItemSource::TaskReward);
        }
        if code == TASKS_COIN {
            sources.push(ItemSource::Task);
        }
        //if [
        //    "blue_candy",
        //    "green_candy",
        //    "red_candy",
        //    "yellow_candy",
        //    "christmas_cane",
        //    "christmas_star",
        //    "frozen_gloves",
        //    "frozen_axe",
        //    "frozen_fishing_rod",
        //    "frozen_pickaxe",
        //]
        //.contains(&code)
        //{
        //    sources.push(ItemSource::Gift);
        //}
        sources
    }

    pub fn time_to_get(&self, item: &str) -> Option<i32> {
        self.sources_of(item)
            .iter()
            .map(|s| match s {
                ItemSource::Resource(_) => 20,
                ItemSource::Monster(m) => m.level * self.drop_rate(item),
                ItemSource::Craft => self
                    .mats_of(item)
                    .iter()
                    .map(|m| self.time_to_get(&m.code).unwrap_or(10000) * m.quantity)
                    .sum(),
                ItemSource::TaskReward => 20000,
                ItemSource::Task => 20000,
                //ItemSource::Gift => 10000,
            })
            .min()
    }

    pub fn is_from_event(&self, code: &str) -> bool {
        self.get(code).is_some_and(|i| {
            self.sources_of(&i.code).iter().any(|s| match s {
                ItemSource::Resource(r) => self.resources.is_event(&r.code),
                ItemSource::Monster(m) => self.monsters.is_event(&m.code),
                ItemSource::Craft => false,
                ItemSource::TaskReward => false,
                ItemSource::Task => false,
                //ItemSource::Gift => false,
            })
        })
    }
}

pub trait ItemSchemaExt {
    fn name(&self) -> String;
    fn r#type(&self) -> Type;
    fn is_of_type(&self, r#type: Type) -> bool;
    fn is_craftable(&self) -> bool;
    fn is_crafted_with(&self, item: &str) -> bool;
    fn is_crafted_from_task(&self) -> bool;
    fn mats(&self) -> Vec<SimpleItemSchema>;
    fn mats_quantity(&self) -> i32;
    fn recycled_quantity(&self) -> i32;
    fn craft_schema(&self) -> Option<CraftSchema>;
    fn skill_to_craft(&self) -> Option<Skill>;
    fn effects(&self) -> Vec<&SimpleEffectSchema>;
    fn attack_damage(&self, r#type: DamageType) -> i32;
    fn attack_damage_against(&self, monster: &MonsterSchema) -> f32;
    fn damage_increase(&self, r#type: DamageType) -> i32;
    fn resistance(&self, r#type: DamageType) -> i32;
    fn health(&self) -> i32;
    fn haste(&self) -> i32;
    fn is_tool(&self) -> bool;
    fn skill_cooldown_reduction(&self, skijll: Skill) -> i32;
    fn heal(&self) -> i32;
    fn restore(&self) -> i32;
    fn inventory_space(&self) -> i32;
    fn is_consumable(&self) -> bool;
    fn is_consumable_at(&self, level: i32) -> bool;
    fn damage_increase_against_with(&self, monster: &MonsterSchema, weapon: &ItemSchema) -> f32;
    fn damage_reduction_against(&self, monster: &MonsterSchema) -> f32;
}

impl ItemSchemaExt for ItemSchema {
    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn r#type(&self) -> Type {
        Type::from_str(&self.r#type).expect("type to be valid")
    }

    fn is_of_type(&self, r#type: Type) -> bool {
        self.r#type == r#type
    }

    fn is_crafted_with(&self, item: &str) -> bool {
        self.mats().iter().any(|m| m.code == item)
    }

    fn is_crafted_from_task(&self) -> bool {
        self.is_crafted_with(JASPER_CRYSTAL)
            || self.is_crafted_with(MAGICAL_CURE)
            || self.is_crafted_with(ENCHANTED_FABRIC)
            || self.is_crafted_with(ASTRALYTE_CRYSTAL)
            || self.is_crafted_with(DIAMOND)
            || self.is_crafted_with("rosenblood_elixir")
            || self.is_crafted_with("hellhound_hair")
            || self.is_crafted_with("efreet_cloth")
    }

    fn mats(&self) -> Vec<SimpleItemSchema> {
        self.craft_schema()
            .into_iter()
            .filter_map(|i| i.items)
            .flatten()
            .collect_vec()
    }

    fn mats_quantity(&self) -> i32 {
        self.mats().iter().map(|m| m.quantity).sum()
    }

    fn recycled_quantity(&self) -> i32 {
        let q = self.mats_quantity();
        q / 5 + if q % 5 > 0 { 1 } else { 0 }
    }

    fn craft_schema(&self) -> Option<CraftSchema> {
        self.craft.clone()?.map(|c| (*c))
    }

    fn skill_to_craft(&self) -> Option<Skill> {
        self.craft_schema()
            .and_then(|schema| schema.skill)
            .map(Skill::from)
    }

    fn effects(&self) -> Vec<&SimpleEffectSchema> {
        self.effects.iter().flatten().collect_vec()
    }

    fn attack_damage(&self, r#type: DamageType) -> i32 {
        self.effects()
            .iter()
            .find(|e| e.code == "attack_".to_string() + r#type.as_ref())
            .map(|e| e.value)
            .unwrap_or(0)
    }

    fn resistance(&self, r#type: DamageType) -> i32 {
        self.effects()
            .iter()
            .find(|e| e.code == "res_".to_string() + r#type.as_ref())
            .map(|e| e.value)
            .unwrap_or(0)
    }

    fn attack_damage_against(&self, monster: &MonsterSchema) -> f32 {
        DamageType::iter()
            .map(|t| Simulator::average_dmg(self.attack_damage(t), 0, monster.resistance(t)))
            .sum()
    }

    fn damage_increase(&self, r#type: DamageType) -> i32 {
        self.effects()
            .iter()
            .find(|e| {
                e.code == "dmg_".to_string() + r#type.as_ref()
                    || e.code == "boost_dmg_".to_string() + r#type.as_ref()
            })
            .map(|e| e.value)
            .unwrap_or(0)
    }

    fn health(&self) -> i32 {
        self.effects()
            .iter()
            .find(|e| e.code == "hp" || e.code == "boost_hp")
            .map(|e| e.value)
            .unwrap_or(0)
    }

    fn haste(&self) -> i32 {
        self.effects()
            .iter()
            .find(|e| e.code == "haste")
            .map(|e| e.value)
            .unwrap_or(0)
    }

    fn is_tool(&self) -> bool {
        Skill::iter().any(|s| self.skill_cooldown_reduction(s) < 0)
    }

    fn skill_cooldown_reduction(&self, skill: Skill) -> i32 {
        self.effects()
            .iter()
            .find_map(|e| (e.code == skill.as_ref()).then_some(e.value))
            .unwrap_or(0)
    }

    fn heal(&self) -> i32 {
        self.effects()
            .iter()
            .find_map(|e| (e.code == "heal").then_some(e.value))
            .unwrap_or(0)
    }

    fn restore(&self) -> i32 {
        self.effects()
            .iter()
            .find_map(|e| (e.code == "restore").then_some(e.value))
            .unwrap_or(0)
    }

    fn inventory_space(&self) -> i32 {
        self.effects()
            .iter()
            .find_map(|e| (e.code == "inventory_space").then_some(e.value))
            .unwrap_or(0)
    }

    fn is_consumable(&self) -> bool {
        self.is_of_type(Type::Consumable)
    }

    fn is_consumable_at(&self, level: i32) -> bool {
        self.is_of_type(Type::Consumable)
            && self.heal() > 0
            && self.level <= level
            && !FOOD_BLACK_LIST.contains(&self.code.as_str())
    }

    fn damage_increase_against_with(&self, monster: &MonsterSchema, weapon: &ItemSchema) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    weapon.attack_damage(t),
                    self.damage_increase(t),
                    monster.resistance(t),
                )
            })
            .sum::<f32>()
            - DamageType::iter()
                .map(|t| Simulator::average_dmg(weapon.attack_damage(t), 0, monster.resistance(t)))
                .sum::<f32>()
    }

    fn damage_reduction_against(&self, monster: &MonsterSchema) -> f32 {
        DamageType::iter()
            .map(|t| Simulator::average_dmg(monster.attack_damage(t), 0, 0))
            .sum::<f32>()
            - DamageType::iter()
                .map(|t| Simulator::average_dmg(monster.attack_damage(t), 0, self.resistance(t)))
                .sum::<f32>()
    }

    fn is_craftable(&self) -> bool {
        self.craft_schema().is_some()
    }
}

impl fmt::Display for dyn ItemSchemaExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Display, AsRefStr, EnumIter, EnumString, EnumIs)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    Consumable,
    BodyArmor,
    Weapon,
    Resource,
    LegArmor,
    Helmet,
    Boots,
    Shield,
    Amulet,
    Ring,
    Artifact,
    Currency,
    Utility,
    Bag,
    Rune,
}

impl From<Slot> for Type {
    fn from(value: Slot) -> Self {
        match value {
            Slot::Weapon => Self::Weapon,
            Slot::Shield => Self::Shield,
            Slot::Helmet => Self::Helmet,
            Slot::BodyArmor => Self::BodyArmor,
            Slot::LegArmor => Self::LegArmor,
            Slot::Boots => Self::Boots,
            Slot::Ring1 => Self::Ring,
            Slot::Ring2 => Self::Ring,
            Slot::Amulet => Self::Amulet,
            Slot::Artifact1 => Self::Artifact,
            Slot::Artifact2 => Self::Artifact,
            Slot::Artifact3 => Self::Artifact,
            Slot::Utility1 => Self::Utility,
            Slot::Utility2 => Self::Utility,
            Slot::Bag => Self::Bag,
            Slot::Rune => Self::Rune,
        }
    }
}

impl PartialEq<Type> for String {
    fn eq(&self, other: &Type) -> bool {
        other.as_ref() == *self
    }
}

#[derive(Debug, PartialEq, AsRefStr, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum SubType {
    Mining,
    Woodcutting,
    Fishing,
    Food,
    Bar,
    Plank,
    Mob,
}

impl PartialEq<SubType> for String {
    fn eq(&self, other: &SubType) -> bool {
        other.as_ref() == *self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, AsRefStr, EnumIter, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum DamageType {
    Air,
    Earth,
    Fire,
    Water,
}

#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum ItemSource {
    Resource(Arc<ResourceSchema>),
    Monster(Arc<MonsterSchema>),
    Craft,
    TaskReward,
    Task,
    //Gift,
}

#[cfg(test)]
mod tests {
    //TODO: rewrite test
    // use itertools::Itertools;
    // use crate::{items::ItemSchemaExt, };

    // #[test]
    // fn item_damage_against() {
    //     assert_eq!(
    //         ITEMS
    //             .get("skull_staff")
    //             .unwrap()
    //             .attack_damage_against(&MONSTERS.get("ogre").unwrap()),
    //         48.0
    //     );
    //     assert_eq!(
    //         ITEMS
    //             .get("dreadful_staff")
    //             .unwrap()
    //             .attack_damage_against(&MONSTERS.get("vampire").unwrap()),
    //         57.5
    //     );
    // }
    //
    // #[test]
    // fn damage_increase() {
    //     assert_eq!(
    //         ITEMS
    //             .get("steel_boots")
    //             .unwrap()
    //             .damage_increase(super::DamageType::Air),
    //         0
    //     )
    // }
    //
    // #[test]
    // fn damage_increase_against() {
    //     assert_eq!(
    //         ITEMS
    //             .get("steel_armor")
    //             .unwrap()
    //             .damage_increase_against_with(
    //                 &MONSTERS.get("chicken").unwrap(),
    //                 &ITEMS.get("steel_battleaxe").unwrap()
    //             ),
    //         6.0
    //     );
    //
    //     assert_eq!(
    //         ITEMS
    //             .get("steel_boots")
    //             .unwrap()
    //             .damage_increase_against_with(
    //                 &MONSTERS.get("ogre").unwrap(),
    //                 &ITEMS.get("skull_staff").unwrap()
    //             ),
    //         0.0
    //     );
    // }
    //
    // #[test]
    // fn damage_reduction_against() {
    //     assert_eq!(
    //         ITEMS
    //             .get("steel_armor")
    //             .unwrap()
    //             .damage_reduction_against(&MONSTERS.get("ogre").unwrap()),
    //         4.0
    //     );
    // }
    //
    // //#[test]
    // //fn gift_source() {
    // //    assert_eq!(
    // //        ITEMS.sources_of("christmas_star").first(),
    // //        Some(&ItemSource::Gift)
    // //    );
    // //    assert_eq!(
    // //        ITEMS.best_source_of("gift"),
    // //        Some(&ItemSource::Monster(MONSTERS.get("gingerbread").unwrap())).cloned()
    // //    );
    // //}
    //
    // #[test]
    // fn best_consumable_foods() {
    //     assert_eq!(
    //         ITEMS
    //             .best_consumable_foods(29)
    //             .iter()
    //             .max_by_key(|i| i.heal())
    //             .unwrap()
    //             .code,
    //         "cooked_trout"
    //     );
    // }
    //
    // #[test]
    // fn drop_rate() {
    //     assert_eq!(ITEMS.drop_rate("milk_bucket"), 12);
    // }
    //
    // #[test]
    // fn require_task_reward() {
    //     assert!(ITEMS.require_task_reward("greater_dreadful_staff"));
    // }
    //
    // #[test]
    // fn mats_methods() {
    //     assert!(!ITEMS.mats_of("greater_dreadful_staff").is_empty());
    //     assert!(!ITEMS.base_mats_of("greater_dreadful_staff").is_empty());
    // }
}
