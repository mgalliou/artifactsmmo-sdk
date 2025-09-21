use crate::{DataPage, PaginatedRequest};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        events_api::{get_all_events_events_get, GetAllEventsEventsGetError},
        Error,
    },
    models::{DataPageEventSchema, EventSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct EventsApi {
    configuration: Arc<Configuration>,
}

impl EventsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl PaginatedRequest<EventSchema, DataPageEventSchema, GetAllEventsEventsGetError> for EventsApi {
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageEventSchema, Error<GetAllEventsEventsGetError>> {
        get_all_events_events_get(&self.configuration, None, Some(current_page), Some(100))
    }
}

impl DataPage<EventSchema> for DataPageEventSchema {
    fn data(self) -> Vec<EventSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
