use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        maps_api::{
            get_all_maps_maps_get, get_map_maps_xy_get, GetAllMapsMapsGetError,
            GetMapMapsXyGetError,
        },
        Error,
    },
    models::{DataPageMapSchema, MapResponseSchema, MapSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct MapsApi {
    configuration: Arc<Configuration>,
}

impl MapsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    pub fn get(&self, x: i32, y: i32) -> Result<MapResponseSchema, Error<GetMapMapsXyGetError>> {
        get_map_maps_xy_get(&self.configuration, x, y)
    }
}

impl PaginatedApi<MapSchema, DataPageMapSchema, GetAllMapsMapsGetError> for MapsApi {
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageMapSchema, Error<GetAllMapsMapsGetError>> {
        get_all_maps_maps_get(
            &self.configuration,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<MapSchema> for DataPageMapSchema {
    fn data(self) -> Vec<MapSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
