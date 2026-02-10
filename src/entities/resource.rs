use std::sync::Arc;

use artifactsmmo_openapi::models::{DropRateSchema, ResourceSchema};
use serde::{Deserialize, Serialize};

use crate::{CanProvideXp, Code, DropsItems, Level, Skill};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Resource(Arc<ResourceSchema>);

impl Resource {
    pub fn new(resource: ResourceSchema) -> Self {
        Self(Arc::new(resource))
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn skill(&self) -> Skill {
        self.0.skill.into()
    }
}

impl DropsItems for Resource {
    fn drops(&self) -> &Vec<DropRateSchema> {
        &self.0.drops
    }
}

impl Code for Resource {
    fn code(&self) -> &str {
        &self.0.code
    }
}

impl Level for Resource {
    fn level(&self) -> u32 {
        self.0.level as u32
    }
}

impl CanProvideXp for Resource {}
