use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        events_api::{
            get_all_active_events_events_active_get, GetAllActiveEventsEventsActiveGetError,
        },
        Error,
    },
    models::{ActiveEventSchema, DataPageActiveEventSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct ActiveEventsApi {
    configuration: Arc<Configuration>,
}

impl ActiveEventsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }
}

impl
    PaginatedApi<
        ActiveEventSchema,
        DataPageActiveEventSchema,
        GetAllActiveEventsEventsActiveGetError,
    > for ActiveEventsApi
{
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageActiveEventSchema, Error<GetAllActiveEventsEventsActiveGetError>> {
        get_all_active_events_events_active_get(&self.configuration, Some(current_page), Some(100))
    }
}

impl DataPage<ActiveEventSchema> for DataPageActiveEventSchema {
    fn data(self) -> Vec<ActiveEventSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
