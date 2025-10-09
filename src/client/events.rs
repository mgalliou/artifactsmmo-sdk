use crate::{DataItem, Persist, maps::MapSchemaExt};
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{ActiveEventSchema, EventSchema};
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use log::debug;
use sdk_derive::CollectionClient;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Default, Debug, CollectionClient)]
pub struct EventsClient {
    data: RwLock<HashMap<String, Arc<EventSchema>>>,
    api: Arc<ArtifactApi>,
    active: RwLock<Vec<Arc<ActiveEventSchema>>>,
    last_refresh: RwLock<DateTime<Utc>>,
}

impl EventsClient {
    pub(crate) fn new(api: Arc<ArtifactApi>) -> Self {
        let events = Self {
            data: Default::default(),
            api,
            active: RwLock::new(vec![]),
            last_refresh: RwLock::new(DateTime::<Utc>::MIN_UTC),
        };
        *events.data.write().unwrap() = events.load();
        events.refresh_active();
        events
    }

    pub fn active(&self) -> Vec<Arc<ActiveEventSchema>> {
        self.active.read().unwrap().iter().cloned().collect_vec()
    }

    pub fn refresh_active(&self) {
        let now = Utc::now();
        if Utc::now() - self.last_refresh() <= Duration::seconds(30) {
            return;
        }
        // NOTE: keep `events` locked before updating last refresh
        let mut events = self.active.write().unwrap();
        self.update_last_refresh(now);
        if let Ok(new) = self.api.events.get_active() {
            *events = new.into_iter().map(Arc::new).collect_vec();
            debug!("events refreshed.");
        }
    }

    fn update_last_refresh(&self, now: DateTime<Utc>) {
        self.last_refresh
            .write()
            .expect("`last_refresh` to be writable")
            .clone_from(&now);
    }

    pub fn last_refresh(&self) -> DateTime<Utc> {
        *self
            .last_refresh
            .read()
            .expect("`last_refresh` to be readable")
    }
}

impl Persist<HashMap<String, Arc<EventSchema>>> for EventsClient {
    const PATH: &'static str = ".cache/events.json";

    fn load_from_api(&self) -> HashMap<String, Arc<EventSchema>> {
        self.api
            .events
            .get_all()
            .unwrap()
            .into_iter()
            .map(|event| (event.code.clone(), Arc::new(event)))
            .collect()
    }

    fn refresh(&self) {
        *self.data.write().unwrap() = self.load_from_api();
    }
}

impl DataItem for EventsClient {
    type Item = Arc<EventSchema>;
}

pub trait EventSchemaExt {
    fn content_code(&self) -> &String;
    fn to_string(&self) -> String;
}

impl EventSchemaExt for EventSchema {
    fn content_code(&self) -> &String {
        &self.content.code
    }

    fn to_string(&self) -> String {
        format!("{}: '{}'", self.name, self.content_code())
    }
}

impl EventSchemaExt for ActiveEventSchema {
    fn content_code(&self) -> &String {
        self.map
            .content()
            .map(|c| &c.code)
            .expect("event to have content")
    }

    fn to_string(&self) -> String {
        let remaining = if let Ok(expiration) = DateTime::parse_from_rfc3339(&self.expiration) {
            (expiration.to_utc() - Utc::now()).num_seconds().to_string()
        } else {
            "?".to_string()
        };
        format!(
            "{} ({},{}): '{}', duration: {}, created at {}, expires at {}, remaining: {}s",
            self.name,
            self.map.x,
            self.map.y,
            self.content_code(),
            self.duration,
            self.created_at,
            self.expiration,
            remaining
        )
    }
}
