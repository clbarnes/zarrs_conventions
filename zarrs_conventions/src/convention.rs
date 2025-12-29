use iref::{Uri, UriBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ConventionId;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, PartialOrd, Eq, Ord)]
pub struct ConventionDefinition {
    pub uuid: Uuid,
    pub schema_url: &'static Uri,
    pub spec_url: &'static Uri,
    pub name: &'static str,
    pub description: &'static str,
}

impl ConventionDefinition {
    pub fn id_uuid(&self) -> ConventionId {
        ConventionId::Uuid(self.uuid)
    }
    pub fn id_schema(&self) -> ConventionId {
        ConventionId::SchemaUrl(self.schema_url.to_owned())
    }
    pub fn id_spec(&self) -> ConventionId {
        ConventionId::SpecUrl(self.spec_url.to_owned())
    }
}

impl From<ConventionDefinition> for Convention {
    fn from(def: ConventionDefinition) -> Self {
        Convention {
            uuid: Some(def.uuid),
            schema_url: Some(def.schema_url.to_owned()),
            spec_url: Some(def.spec_url.to_owned()),
            name: Some(def.name.to_string()),
            description: Some(def.description.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialOrd, Ord, PartialEq, Eq)]
pub struct Convention {
    pub(crate) uuid: Option<Uuid>,
    pub(crate) schema_url: Option<UriBuf>,
    pub(crate) spec_url: Option<UriBuf>,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
}

impl<'de> Deserialize<'de> for Convention {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bld = ConventionBuilder::deserialize(deserializer)?;
        bld.build().map_err(serde::de::Error::custom)
    }
}

impl Convention {
    pub fn builder() -> ConventionBuilder {
        ConventionBuilder::default()
    }

    pub fn id(&self) -> ConventionId {
        if let Some(uuid) = self.uuid {
            ConventionId::Uuid(uuid)
        } else if let Some(ref url) = self.schema_url {
            ConventionId::SchemaUrl(url.clone())
        } else if let Some(ref url) = self.spec_url {
            ConventionId::SpecUrl(url.clone())
        } else {
            unreachable!("Convention must have at least one identifier");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConventionBuilder {
    uuid: Option<Uuid>,
    schema_url: Option<UriBuf>,
    spec_url: Option<UriBuf>,
    name: Option<String>,
    description: Option<String>,
}

impl ConventionBuilder {
    pub fn uuid(mut self, uuid: Uuid) -> Self {
        self.uuid = Some(uuid);
        self
    }

    pub fn schema_url<U: Into<UriBuf>>(mut self, url: U) -> Self {
        self.schema_url = Some(url.into());
        self
    }

    pub fn spec_url<U: Into<UriBuf>>(mut self, url: U) -> Self {
        self.spec_url = Some(url.into());
        self
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn build(self) -> Result<Convention, String> {
        if self.uuid.is_none() && self.schema_url.is_none() && self.spec_url.is_none() {
            return Err("At least one of uuid, schema_url, or spec_url must be set".to_string());
        }
        Ok(Convention {
            uuid: self.uuid,
            schema_url: self.schema_url,
            spec_url: self.spec_url,
            name: self.name,
            description: self.description,
        })
    }
}
