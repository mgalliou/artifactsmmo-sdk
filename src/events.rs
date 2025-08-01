use crate::PersistedData;
use artifactsmmo_api_wrapper::ArtifactApi;
use artifactsmmo_openapi::models::{ActiveEventSchema, EventSchema};
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use log::debug;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Events {
    data: RwLock<Vec<Arc<EventSchema>>>,
    api: Arc<ArtifactApi>,
    active: RwLock<Vec<Arc<ActiveEventSchema>>>,
    last_refresh: RwLock<DateTime<Utc>>,
}

impl PersistedData<Vec<Arc<EventSchema>>> for Events {
    const PATH: &'static str = ".cache/events.json";

    async fn data_from_api(&self) -> Vec<Arc<EventSchema>> {
        self.api
            .events
            .all()
            .await
            .unwrap()
            .into_iter()
            .map(Arc::new)
            .collect()
    }

    async fn refresh_data(&self) {
        *self.data.write().await = self.data_from_api().await;
    }
}

impl Events {
    pub(crate) async fn new(api: Arc<ArtifactApi>) -> Self {
        let events = Self {
            data: RwLock::new(vec![]),
            api,
            active: RwLock::new(vec![]),
            last_refresh: RwLock::new(DateTime::<Utc>::MIN_UTC),
        };
        *events.data.write().await = events.retrieve_data().await;
        events.refresh_active().await;
        events
    }

    pub async fn all(&self) -> Vec<Arc<EventSchema>> {
        self.data
            .read()
            .await
            .iter()
            .cloned()
            .collect_vec()
    }

    pub async fn active(&self) -> Vec<Arc<ActiveEventSchema>> {
        self.active
            .read()
            .await
            .iter()
            .cloned()
            .collect_vec()
    }

    pub async fn refresh_active(&self) {
        let now = Utc::now();
        if Utc::now() - self.last_refresh().await <= Duration::seconds(30) {
            return;
        }
        // NOTE: keep `events` locked before updating last refresh
        let mut events = self.active.write().await;
        self.update_last_refresh(now);
        if let Ok(new) = self.api.events.active().await {
            *events = new.into_iter().map(Arc::new).collect_vec();
            debug!("events refreshed.");
        }
    }

    async fn update_last_refresh(&self, now: DateTime<Utc>) {
        self.last_refresh
            .write()
            .await
            .clone_from(&now);
    }

    pub async fn last_refresh(&self) -> DateTime<Utc> {
        *self
            .last_refresh
            .read()
            .await
    }
}

impl EventSchemaExt for ActiveEventSchema {
    fn content_code(&self) -> &String {
        self.map
            .content
            .as_ref()
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
