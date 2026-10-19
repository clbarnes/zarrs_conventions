//! Types used only for (de)serialisation.
use serde::{Deserialize, Serialize};

use crate::{
    Configuration, DEFAULT_DATA_TYPE_NAME, FillValue, InlineArray, InlineArrayBuilder, Metadata,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum MetadataRaw {
    Name(String),
    Config {
        name: String,
        #[serde(default)]
        configuration: Configuration,
    },
}

impl From<MetadataRaw> for Metadata {
    fn from(value: MetadataRaw) -> Self {
        match value {
            MetadataRaw::Name(name) => Metadata {
                name,
                configuration: Default::default(),
            },
            MetadataRaw::Config {
                name,
                configuration,
            } => Metadata {
                name,
                configuration,
            },
        }
    }
}

impl From<Metadata> for MetadataRaw {
    fn from(meta: Metadata) -> Self {
        if meta.configuration.is_empty() {
            MetadataRaw::Name(meta.name)
        } else {
            MetadataRaw::Config {
                name: meta.name,
                configuration: meta.configuration,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InlineArrayRaw {
    values: Vec<FillValue>,
    data_type: Option<Metadata>,
    shape: Option<Vec<usize>>,
    dimension_names: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    attributes: serde_json::Map<String, serde_json::Value>,
}

impl TryFrom<InlineArrayRaw> for InlineArray {
    type Error = crate::Error;

    fn try_from(value: InlineArrayRaw) -> Result<Self, Self::Error> {
        let mut b = InlineArrayBuilder::new(value.values);
        if let Some(dt) = value.data_type {
            b = b.with_data_type(dt);
        }
        if let Some(shape) = value.shape {
            b = b.with_shape(shape)?;
        }
        if let Some(dimension_names) = value.dimension_names {
            b = b.dimension_names(dimension_names)?;
        }

        b.attributes_mut().extend(value.attributes);
        b.try_into()
    }
}

impl From<InlineArray> for InlineArrayRaw {
    fn from(value: InlineArray) -> Self {
        let data_type = if value.data_type.name == DEFAULT_DATA_TYPE_NAME
            && value.data_type.configuration.is_empty()
        {
            None
        } else {
            Some(value.data_type)
        };

        let shape = if value.shape.len() == 1 {
            None
        } else {
            Some(value.shape)
        };

        InlineArrayRaw {
            values: value.values,
            data_type,
            shape,
            dimension_names: value.dimension_names,
            attributes: value.attributes,
        }
    }
}
