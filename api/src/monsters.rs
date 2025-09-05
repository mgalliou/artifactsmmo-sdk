use crate::{DataPage, PaginatedApi};
use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        monsters_api::{
            get_all_monsters_monsters_get, get_monster_monsters_code_get,
            GetAllMonstersMonstersGetError, GetMonsterMonstersCodeGetError,
        },
        Error,
    },
    models::{DataPageMonsterSchema, MonsterResponseSchema, MonsterSchema},
};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct MonstersApi {
    configuration: Arc<Configuration>,
}

impl MonstersApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    pub fn get(
        &self,
        code: &str,
    ) -> Result<MonsterResponseSchema, Error<GetMonsterMonstersCodeGetError>> {
        get_monster_monsters_code_get(&self.configuration, code)
    }
}

impl PaginatedApi<MonsterSchema, DataPageMonsterSchema, GetAllMonstersMonstersGetError>
    for MonstersApi
{
    fn api_call(
        &self,
        current_page: u32,
    ) -> Result<DataPageMonsterSchema, Error<GetAllMonstersMonstersGetError>> {
        get_all_monsters_monsters_get(
            &self.configuration,
            None,
            None,
            None,
            None,
            Some(current_page),
            Some(100),
        )
    }
}

impl DataPage<MonsterSchema> for DataPageMonsterSchema {
    fn data(self) -> Vec<MonsterSchema> {
        self.data
    }

    fn pages(&self) -> Option<Option<u32>> {
        self.pages
    }
}
