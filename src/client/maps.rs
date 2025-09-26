use crate::{client::events::EventsClient, skill::Skill};
use artifactsmmo_api_wrapper::{ArtifactApi};
use artifactsmmo_openapi::models::{MapContentSchema, MapContentType, MapSchema, TaskType};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug)]
pub struct MapsClient {
    data: HashMap<(i32, i32), RwLock<Arc<MapSchema>>>,
    events: Arc<EventsClient>,
}

impl MapsClient {
    pub(crate) fn new(api: &ArtifactApi, events: Arc<EventsClient>) -> Self {
        Self {
            data: api
                .maps
                .get_all()
                .unwrap()
                .into_iter()
                .map(|m| ((m.x, m.y), RwLock::new(Arc::new(m))))
                .collect(),
            events,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Arc<MapSchema>> {
        Some(self.data.get(&(x, y))?.read().unwrap().clone())
    }

    pub fn refresh_from_events(&self) {
        self.events.active().iter().for_each(|e| {
            if DateTime::parse_from_rfc3339(&e.expiration).unwrap() < Utc::now()
                && let Some(map) = self.data.get(&(e.map.x, e.map.y))
            {
                *map.write().unwrap() = Arc::new(*e.previous_map.clone())
            }
        });
        self.events.refresh_active();
        self.events.active().iter().for_each(|e| {
            if DateTime::parse_from_rfc3339(&e.expiration).unwrap() > Utc::now()
                && let Some(map) = self.data.get(&(e.map.x, e.map.y))
            {
                *map.write().unwrap() = Arc::new(*e.map.clone())
            }
        });
    }

    pub fn closest_from_amoung(x: i32, y: i32, maps: &[Arc<MapSchema>]) -> Option<Arc<MapSchema>> {
        maps.iter()
            .min_by_key(|m| i32::abs(x - m.x) + i32::abs(y - m.y))
            .cloned()
    }

    pub fn of_type(&self, r#type: MapContentType) -> Vec<Arc<MapSchema>> {
        self.data
            .values()
            .filter_map(|m| {
                let map = m.read().unwrap().clone();
                map.content_type_is(r#type).then_some(map)
            })
            .collect_vec()
    }

    pub fn with_content_code(&self, code: &str) -> Vec<Arc<MapSchema>> {
        self.data
            .values()
            .filter_map(|m| {
                let map = m.read().unwrap().clone();
                map.content_code_is(code).then_some(map)
            })
            .collect()
    }

    pub fn with_content(&self, content: &MapContentSchema) -> Vec<Arc<MapSchema>> {
        self.data
            .values()
            .filter_map(|m| {
                let map = m.read().unwrap().clone();
                map.content_is(content).then_some(map)
            })
            .collect()
    }

    pub fn with_workshop_for(&self, skill: Skill) -> Option<Arc<MapSchema>> {
        match skill {
            Skill::Weaponcrafting
            | Skill::Gearcrafting
            | Skill::Jewelrycrafting
            | Skill::Cooking
            | Skill::Woodcutting
            | Skill::Mining
            | Skill::Alchemy => self.with_content_code(skill.as_ref()).first().cloned(),
            Skill::Combat => None,
            Skill::Fishing => None,
        }
    }

    pub fn closest_with_content_code_from(
        &self,
        map: Arc<MapSchema>,
        code: &str,
    ) -> Option<Arc<MapSchema>> {
        let maps = self.with_content_code(code);
        if maps.is_empty() {
            return None;
        }
        map.closest_among(&maps)
    }

    fn closest_with_content_from(
        &self,
        map: Arc<MapSchema>,
        content: &MapContentSchema,
    ) -> Option<Arc<MapSchema>> {
        let maps = self.with_content(content);
        if maps.is_empty() {
            return None;
        }
        map.closest_among(&maps)
    }

    pub fn closest_of_type_from(
        &self,
        map: Arc<MapSchema>,
        r#type: MapContentType,
    ) -> Option<Arc<MapSchema>> {
        let maps = self.of_type(r#type);
        if maps.is_empty() {
            return None;
        }
        map.closest_among(&maps)
    }

    pub fn closest_tasksmaster_from(
        &self,
        map: Arc<MapSchema>,
        r#type: Option<TaskType>,
    ) -> Option<Arc<MapSchema>> {
        if let Some(r#type) = r#type {
            self.closest_with_content_from(
                map,
                &MapContentSchema {
                    r#type: MapContentType::TasksMaster,
                    code: r#type.to_string(),
                },
            )
        } else {
            self.closest_of_type_from(map, MapContentType::TasksMaster)
        }
    }
}

pub trait MapSchemaExt {
    fn content(&self) -> Option<&MapContentSchema>;
    fn content_is(&self, content: &MapContentSchema) -> bool;
    fn content_code_is(&self, code: &str) -> bool;
    fn content_type_is(&self, r#type: MapContentType) -> bool;
    fn monster(&self) -> Option<String>;
    fn resource(&self) -> Option<String>;
    fn closest_among(&self, others: &[Arc<MapSchema>]) -> Option<Arc<MapSchema>>;
    fn is_tasksmaster(&self, task_type: Option<TaskType>) -> bool;
    fn pretty(&self) -> String;
}

impl MapSchemaExt for MapSchema {
    fn content(&self) -> Option<&MapContentSchema> {
        self.content.as_ref().map(|c| c.as_ref())
    }

    fn content_is(&self, content: &MapContentSchema) -> bool {
        self.content().is_some_and(|c| c == content)
    }

    fn content_code_is(&self, code: &str) -> bool {
        self.content().is_some_and(|c| c.code == code)
    }

    fn content_type_is(&self, r#type: MapContentType) -> bool {
        self.content().is_some_and(|c| c.r#type == r#type)
    }

    fn monster(&self) -> Option<String> {
        Some(self.content()?.code.clone())
    }

    fn resource(&self) -> Option<String> {
        Some(self.content()?.code.clone())
    }

    fn closest_among(&self, others: &[Arc<MapSchema>]) -> Option<Arc<MapSchema>> {
        MapsClient::closest_from_amoung(self.x, self.y, others)
    }

    fn is_tasksmaster(&self, task_type: Option<TaskType>) -> bool {
        self.content_type_is(MapContentType::TasksMaster)
            && task_type.is_none_or(|tt| self.content_code_is(&tt.to_string()))
    }

    fn pretty(&self) -> String {
        if let Some(content) = self.content() {
            format!("{} ({},{} [{}])", self.name, self.x, self.y, content.code)
        } else {
            format!("{} ({},{})", self.name, self.x, self.y)
        }
    }
}
#[cfg(test)]
mod tests {
    //use super::*;

    // #[test]
    // fn check_content_type_as_string() {
    //     assert_eq!(ContentType::Monster.to_string(), "monster");
    //     assert_eq!(ContentType::Monster.as_ref(), "monster");
    // }
}
