use crate::{DataPage, Paginate};
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

    pub fn get_all(&self) -> Result<Vec<MapSchema>, Error<GetAllMapsMapsGetError>> {
        MapsRequest {
            configuration: &self.configuration,
        }
        .send()
    }

    pub fn get(&self, x: i32, y: i32) -> Result<MapResponseSchema, Error<GetMapMapsXyGetError>> {
        get_map_maps_xy_get(&self.configuration, x, y)
    }
}

struct MapsRequest<'a> {
    configuration: &'a Configuration,
}

impl<'a> Paginate for MapsRequest<'a> {
    type Data = MapSchema;
    type Page = DataPageMapSchema;
    type Error = GetAllMapsMapsGetError;

    fn request_page(&self, page: u32) -> Result<Self::Page, Error<Self::Error>> {
        get_all_maps_maps_get(self.configuration, None, None, Some(page), Some(100))
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
