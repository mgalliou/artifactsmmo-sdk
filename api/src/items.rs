use artifactsmmo_openapi::{
    apis::{
        configuration::Configuration,
        items_api::{
            get_all_items_items_get, get_item_items_code_get, GetAllItemsItemsGetError,
            GetItemItemsCodeGetError,
        },
        Error,
    },
    models::{ItemResponseSchema, ItemSchema},
};
use std::sync::Arc;

#[derive(Default)]
pub struct ItemsApi {
    configuration: Arc<Configuration>,
}

impl ItemsApi {
    pub(crate) fn new(configuration: Arc<Configuration>) -> Self {
        Self { configuration }
    }

    pub fn all(&self) -> Result<Vec<ItemSchema>, Error<GetAllItemsItemsGetError>> {
        let mut items: Vec<ItemSchema> = vec![];
        let mut current_page = 1;
        let mut finished = false;
        while !finished {
            let resp = get_all_items_items_get(
                &self.configuration,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(current_page),
                Some(100),
            );
            match resp {
                Ok(resp) => {
                    items.extend(resp.data);
                    if let Some(Some(pages)) = resp.pages {
                        if current_page >= pages {
                            finished = true
                        }
                        current_page += 1;
                    } else {
                        // No pagination information, assume single page
                        finished = true
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(items)
    }

    pub fn info(&self, code: &str) -> Result<ItemResponseSchema, Error<GetItemItemsCodeGetError>> {
        get_item_items_code_get(&self.configuration, code)
    }
}
