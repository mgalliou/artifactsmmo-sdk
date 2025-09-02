use crate::{
    char::Skill, consts::{GEMS, GIFT, GINGERBREAD, TASKS_COIN, TASKS_REWARDS_SPECIFICS}, gear::Slot, monsters::{Monsters}, npcs::Npcs, resources::Resources, simulator::HasEffects, tasks_rewards::TasksRewards, HasDropTable, PersistedData, Simulator
};
use artifactsmmo_api_wrapper::{ArtifactApi, PaginatedApi};
use artifactsmmo_openapi::models::{
    CraftSchema, ItemSchema, MonsterSchema, NpcSchema, NpcType, ResourceSchema, SimpleEffectSchema,
    SimpleItemSchema,
};
use itertools::Itertools;
use std::{
    collections::HashMap,
    fmt,
    ops::Deref,
    str::FromStr,
    sync::{Arc, RwLock},
    vec::{IntoIter, Vec},
};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIs, EnumIter, EnumString};

#[derive(Default, Debug)]
pub struct Items {
    data: RwLock<HashMap<String, Arc<ItemSchema>>>,
    api: Arc<ArtifactApi>,
    resources: Arc<Resources>,
    monsters: Arc<Monsters>,
    tasks_rewards: Arc<TasksRewards>,
    npcs: Arc<Npcs>,
}

impl PersistedData<HashMap<String, Arc<ItemSchema>>> for Items {
    const PATH: &'static str = ".cache/items.json";

    fn data_from_api(&self) -> HashMap<String, Arc<ItemSchema>> {
        self.api
            .items
            .all()
            .unwrap()
            .into_iter()
            .map(|item| (item.code.clone(), Arc::new(item)))
            .collect()
    }

    fn refresh_data(&self) {
        *self.data.write().unwrap() = self.data_from_api();
    }
}

impl IntoIterator for Items {
    type Item = Arc<ItemSchema>;

    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect_vec()
            .into_iter()
    }
}

impl Items {
    pub(crate) fn new(
        api: Arc<ArtifactApi>,
        resources: Arc<Resources>,
        monsters: Arc<Monsters>,
        tasks_rewards: Arc<TasksRewards>,
        npcs: Arc<Npcs>,
    ) -> Self {
        let items = Self {
            data: Default::default(),
            api,
            resources,
            monsters,
            tasks_rewards,
            npcs,
        };
        *items.data.write().unwrap() = items.retrieve_data();
        items
    }

    /// Takes an item `code` and return its schema.
    pub fn get(&self, code: &str) -> Option<Arc<ItemSchema>> {
        self.data.read().unwrap().get(code).cloned()
    }

    pub fn all(&self) -> Vec<Arc<ItemSchema>> {
        self.data.read().unwrap().values().cloned().collect_vec()
    }

    /// Takes an item `code` and return the mats required to craft it.
    pub fn mats_of(&self, code: &str) -> Vec<SimpleItemSchema> {
        self.get(code).iter().flat_map(|i| i.mats()).collect_vec()
    }

    /// Takes an item `code` and return the mats required to craft it.
    pub fn mats_for(&self, code: &str, quantity: i32) -> Vec<SimpleItemSchema> {
        self.mats_of(code)
            .into_iter()
            .update(|m| m.quantity *= quantity)
            .collect_vec()
    }

    /// Takes an item `code` and returns the mats down to the raw materials
    /// required to craft it.
    pub fn base_mats_of(&self, code: &str) -> Vec<SimpleItemSchema> {
        self.mats_of(code)
            .iter()
            .flat_map(|mat| {
                if self.mats_of(&mat.code).is_empty() {
                    vec![SimpleItemSchema {
                        code: mat.code.clone(),
                        quantity: mat.quantity,
                    }]
                } else {
                    self.mats_of(&mat.code)
                        .iter()
                        .map(|b| SimpleItemSchema {
                            code: b.code.clone(),
                            quantity: b.quantity * mat.quantity,
                        })
                        .collect_vec()
                }
            })
            .collect_vec()
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
    pub fn crafted_with(&self, code: &str) -> Vec<Arc<ItemSchema>> {
        self.all()
            .iter()
            .filter(|i| i.is_crafted_with(code))
            .cloned()
            .collect_vec()
    }

    pub fn require_task_reward(&self, code: &str) -> bool {
        self.base_mats_of(code)
            .iter()
            .any(|m| TASKS_REWARDS_SPECIFICS.contains(&m.code.as_str()))
    }

    /// Takes an item `code` and returns the only item it can be crafted in, or
    /// `None` otherwise.
    pub fn unique_craft(&self, code: &str) -> Option<Arc<ItemSchema>> {
        let crafts = self.crafted_with(code);
        if crafts.len() == 1 {
            return Some(crafts[0].clone());
        }
        None
    }

    /// Takes an item `code` and returns the items crafted with it as base mat.
    pub fn crafted_with_base_mat(&self, code: &str) -> Vec<Arc<ItemSchema>> {
        self.all()
            .iter()
            .filter(|i| self.is_crafted_with_base_mat(&i.code, code))
            .cloned()
            .collect_vec()
    }

    /// Takes an item `code` and checks if it is crafted with `mat` as a base
    /// material.
    pub fn is_crafted_with_base_mat(&self, code: &str, mat: &str) -> bool {
        self.base_mats_of(code).iter().any(|m| m.code == mat)
    }

    pub fn mats_mob_average_lvl(&self, code: &str) -> i32 {
        let mob_mats = self
            .mats_of(code)
            .iter()
            .filter_map(|i| self.get(&i.code))
            .filter(|i| i.subtype == SubType::Mob)
            .collect_vec();
        let len = mob_mats.len();
        if len > 0 {
            return mob_mats.iter().map(|i| i.level).sum::<i32>() / mob_mats.len() as i32;
        }
        0
    }

    pub fn mats_mob_max_lvl(&self, code: &str) -> i32 {
        self.mats_of(code)
            .iter()
            .filter_map(|i| self.get(&i.code))
            .filter(|i| i.subtype == SubType::Mob)
            .max_by_key(|i| i.level)
            .map_or(0, |i| i.level)
    }

    /// Takes an item `code` and returns the amount of inventory space the mats
    /// required to craft it are taking.
    pub fn mats_quantity_for(&self, code: &str) -> i32 {
        self.mats_of(code).iter().map(|mat| mat.quantity).sum()
    }

    pub fn recycled_quantity_for(&self, code: &str) -> i32 {
        let mats_quantity_for = self.mats_quantity_for(code);
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
    pub fn base_mats_drop_rate(&self, code: &str) -> f32 {
        let base_mats = self.base_mats_of(code);
        if base_mats.is_empty() {
            return 0.0;
        }
        let base_mats_quantity: i32 = base_mats.iter().map(|m| m.quantity).sum();
        let drop_rate_sum: i32 = base_mats
            .iter()
            .map(|m| self.drop_rate(&m.code) * m.quantity)
            .sum();
        let average: f32 = drop_rate_sum as f32 / base_mats_quantity as f32;
        average
    }

    pub fn restoring_utilities(&self, level: i32) -> Vec<Arc<ItemSchema>> {
        self.all()
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
        if code == GIFT {
            return self.monsters.get(GINGERBREAD).map(ItemSource::Monster);
        }
        if GEMS.contains(&code) {
            return Some(ItemSource::Craft);
        }
        if TASKS_REWARDS_SPECIFICS.contains(&code) {
            return Some(ItemSource::TaskReward);
        }
        let sources = self.sources_of(code);
        if sources.iter().all(|s| s.is_resource() || s.is_monster()) {
            let bests = sources.into_iter().min_set_by_key(|s| {
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
        sources.extend(
            self.npcs
                .selling(code)
                .into_iter()
                .map(ItemSource::Npc)
                .collect_vec(),
        );
        if self.get(code).is_some_and(|i| i.is_craftable()) {
            sources.push(ItemSource::Craft);
        }
        if self.tasks_rewards.all().iter().any(|r| r.code == code) {
            sources.push(ItemSource::TaskReward);
        }
        if code == TASKS_COIN {
            sources.push(ItemSource::Task);
        }
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
                ItemSource::Npc(_) => 60,
            })
            .min()
    }

    pub fn is_from_event(&self, code: &str) -> bool {
        self.get(code).is_some_and(|i| {
            self.sources_of(&i.code).iter().any(|s| match s {
                ItemSource::Resource(r) => self.resources.is_event(&r.code),
                ItemSource::Monster(m) => self.monsters.is_event(&m.code),
                ItemSource::Npc(n) => n.r#type == NpcType::Merchant,
                ItemSource::Craft => false,
                ItemSource::TaskReward => false,
                ItemSource::Task => false,
            })
        })
    }
}

pub trait ItemSchemaExt {
    fn is_crafted_with(&self, item: &str) -> bool;
    fn mats_quantity(&self) -> i32;
    fn mats(&self) -> Vec<SimpleItemSchema>;
    fn recycled_quantity(&self) -> i32;
    fn skill_to_craft(&self) -> Option<Skill>;
    fn is_crafted_from_task(&self) -> bool;
    fn is_craftable(&self) -> bool;
    fn craft_schema(&self) -> Option<&CraftSchema>;

    fn attack_damage_against(&self, monster: &MonsterSchema) -> f32;
    fn damage_increase_against_with(&self, monster: &MonsterSchema, weapon: &ItemSchema) -> f32;
    fn damage_reduction_against(&self, monster: &MonsterSchema) -> f32;

    fn is_food(&self) -> bool;
    fn is_consumable(&self) -> bool;
    fn is_tool(&self) -> bool;

    fn is_of_type(&self, r#type: Type) -> bool;
    fn r#type(&self) -> Type;

    fn name(&self) -> String;
}

impl ItemSchemaExt for ItemSchema {
    fn is_crafted_with(&self, item: &str) -> bool {
        self.mats().iter().any(|m| m.code == item)
    }

    fn mats_quantity(&self) -> i32 {
        self.mats().iter().map(|m| m.quantity).sum()
    }

    fn recycled_quantity(&self) -> i32 {
        let q = self.mats_quantity();
        q / 5 + if q % 5 > 0 { 1 } else { 0 }
    }

    fn mats(&self) -> Vec<SimpleItemSchema> {
        self.craft_schema()
            .iter()
            .filter_map(|i| i.items.clone())
            .flatten()
            .collect_vec()
    }

    fn skill_to_craft(&self) -> Option<Skill> {
        self.craft_schema()
            .and_then(|schema| schema.skill)
            .map(Skill::from)
    }

    fn is_crafted_from_task(&self) -> bool {
        TASKS_REWARDS_SPECIFICS
            .iter()
            .any(|i| self.is_crafted_with(i))
    }

    fn is_craftable(&self) -> bool {
        self.craft_schema().is_some()
    }

    fn craft_schema(&self) -> Option<&CraftSchema> {
        self.craft.iter().flatten().map(|c| c.deref()).next_back()
    }

    fn attack_damage_against(&self, monster: &MonsterSchema) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    self.attack_damage(t),
                    0,
                    self.critical_strike(),
                    monster.resistance(t),
                )
            })
            .sum()
    }

    fn damage_increase_against_with(&self, monster: &MonsterSchema, weapon: &ItemSchema) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(
                    weapon.attack_damage(t),
                    self.damage_increase(t),
                    weapon.critical_strike() + self.critical_strike(),
                    monster.resistance(t),
                )
            })
            .sum::<f32>()
            - DamageType::iter()
                .map(|t| {
                    Simulator::average_dmg(
                        weapon.attack_damage(t),
                        0,
                        weapon.critical_strike(),
                        monster.resistance(t),
                    )
                })
                .sum::<f32>()
    }

    fn damage_reduction_against(&self, monster: &MonsterSchema) -> f32 {
        DamageType::iter()
            .map(|t| {
                Simulator::average_dmg(monster.attack_damage(t), 0, monster.critical_strike(), 0)
            })
            .sum::<f32>()
            - DamageType::iter()
                .map(|t| {
                    Simulator::average_dmg(
                        monster.attack_damage(t),
                        0,
                        monster.critical_strike(),
                        self.resistance(t),
                    )
                })
                .sum::<f32>()
    }

    fn is_food(&self) -> bool {
        self.is_consumable() && self.heal() > 0
    }

    fn is_consumable(&self) -> bool {
        self.is_of_type(Type::Consumable)
    }

    fn is_tool(&self) -> bool {
        Skill::iter().any(|s| self.skill_cooldown_reduction(s) < 0)
    }

    fn is_of_type(&self, r#type: Type) -> bool {
        self.r#type == r#type
    }

    fn r#type(&self) -> Type {
        Type::from_str(&self.r#type).expect("type to be valid")
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl HasEffects for ItemSchema {
    fn effects(&self) -> Vec<&SimpleEffectSchema> {
        self.effects.iter().flatten().collect_vec()
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
    Fire,
    Earth,
    Water,
    Air,
}

impl DamageType {
    pub fn into_attack(&self) -> &'static str {
        match self {
            DamageType::Fire => "attack_fire",
            DamageType::Earth => "attack_earth",
            DamageType::Water => "attack_water",
            DamageType::Air => "attack_air",
        }
    }

    pub fn into_damage(&self) -> &'static str {
        match self {
            DamageType::Fire => "dmg_fire",
            DamageType::Earth => "dmg_earth",
            DamageType::Water => "dmg_water",
            DamageType::Air => "dmg_air",
        }
    }

    pub fn into_boost_damage(&self) -> &'static str {
        match self {
            DamageType::Fire => "boost_dmg_fire",
            DamageType::Earth => "boost_dmg_earth",
            DamageType::Water => "boost_dmg_water",
            DamageType::Air => "boost_dmg_air",
        }
    }

    pub fn into_resistance(&self) -> &'static str {
        match self {
            DamageType::Fire => "res_fire",
            DamageType::Earth => "res_earth",
            DamageType::Water => "res_water",
            DamageType::Air => "res_air",
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum ItemSource {
    Resource(Arc<ResourceSchema>),
    Monster(Arc<MonsterSchema>),
    Npc(Arc<NpcSchema>),
    Craft,
    TaskReward,
    Task,
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
