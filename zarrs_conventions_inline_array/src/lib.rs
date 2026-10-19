#![doc=include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, ZarrConventionImpl, iref::uri, register_zarr_conventions, uuid::uuid,
};
mod errors;
mod serde_repr;
pub use errors::{Error, Result};
#[cfg(feature = "zarrs")]
mod zarrs_impl;

pub type Configuration = serde_json::Map<String, serde_json::Value>;
pub type FillValue = serde_json::Value;

const DEFAULT_DATA_TYPE_NAME: &str = "float64";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "serde_repr::MetadataRaw", into = "serde_repr::MetadataRaw")]
pub struct Metadata {
    name: String,
    configuration: Configuration,
}

fn default_data_type() -> Metadata {
    Metadata {
        name: DEFAULT_DATA_TYPE_NAME.to_string(),
        configuration: Default::default(),
    }
}

impl Metadata {
    pub fn new(name: impl Into<String>) -> Self {
        Metadata {
            name: name.into(),
            configuration: Default::default(),
        }
    }

    pub fn with_configuration<C: Serialize>(name: impl Into<String>, configuration: &C) -> Self {
        let val = serde_json::to_value(configuration).expect("Failed to serialize configuration");
        let map = match val {
            serde_json::Value::Object(map) => map,
            _ => panic!("Configuration must serialize to a JSON object"),
        };
        Metadata {
            name: name.into(),
            configuration: map,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    try_from = "serde_repr::InlineArrayRaw",
    into = "serde_repr::InlineArrayRaw"
)]
pub struct InlineArray {
    values: Vec<FillValue>,
    data_type: Metadata,
    shape: Vec<usize>,
    dimension_names: Option<Vec<String>>,
    attributes: serde_json::Map<String, serde_json::Value>,
}

impl InlineArray {
    pub fn builder<V: Serialize, I: IntoIterator<Item = V>>(
        values: I,
    ) -> Result<InlineArrayBuilder, serde_json::Error> {
        InlineArrayBuilder::from_values(values)
    }

    pub fn values(&self) -> &[FillValue] {
        &self.values
    }

    pub fn data_type(&self) -> &Metadata {
        &self.data_type
    }

    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    pub fn dimension_names(&self) -> Option<&[String]> {
        self.dimension_names.as_deref()
    }

    pub fn attributes(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.attributes
    }
}

pub struct InlineArrayBuilder {
    values: Vec<FillValue>,
    data_type: Option<Metadata>,
    shape: Option<Vec<usize>>,
    dimension_names: Option<Vec<String>>,
    attributes: serde_json::Map<String, serde_json::Value>,
}

impl InlineArrayBuilder {
    pub fn new(values: Vec<FillValue>) -> Self {
        Self {
            values,
            data_type: None,
            shape: None,
            dimension_names: None,
            attributes: Default::default(),
        }
    }

    pub fn from_values<V: Serialize, I: IntoIterator<Item = V>>(
        values: I,
    ) -> Result<Self, serde_json::Error> {
        let vals: Result<_, _> = values
            .into_iter()
            .map(|v| serde_json::to_value(v))
            .collect();
        Ok(Self::new(vals?))
    }

    pub fn with_data_type(mut self, data_type: Metadata) -> Self {
        self.data_type = Some(data_type);
        self
    }

    pub fn with_shape(mut self, shape: Vec<usize>) -> Result<Self> {
        let prod: usize = shape.iter().product();
        if prod != self.values.len() {
            return Err(format!(
                "Shape product {} does not match number of values {}",
                prod,
                self.values.len()
            ));
        }
        if let Some(dn) = self.dimension_names.as_ref()
            && dn.len() != shape.len()
        {
            return Err(format!(
                "Dimension names length {} does not match shape length {}",
                dn.len(),
                shape.len()
            ));
        }

        self.shape = Some(shape);
        Ok(self)
    }

    pub fn dimension_names(mut self, dimension_names: Vec<String>) -> Result<Self> {
        if let Some(shape) = self.shape.as_ref()
            && dimension_names.len() != shape.len()
        {
            return Err(format!(
                "Dimension names length {} does not match shape length {}",
                dimension_names.len(),
                shape.len()
            ));
        }

        self.dimension_names = Some(dimension_names);
        Ok(self)
    }

    pub fn with_attribute<V: Serialize>(
        mut self,
        key: impl Into<String>,
        value: &V,
    ) -> Result<Self, serde_json::Error> {
        self.attributes
            .insert(key.into(), serde_json::to_value(value)?);
        Ok(self)
    }

    pub fn attributes_mut(&mut self) -> &mut serde_json::Map<String, serde_json::Value> {
        &mut self.attributes
    }
}

impl TryFrom<InlineArrayBuilder> for InlineArray {
    fn try_from(builder: InlineArrayBuilder) -> Result<Self, Self::Error> {
        let shape = builder.shape.unwrap_or_else(|| vec![builder.values.len()]);
        let data_type = builder.data_type.unwrap_or_else(default_data_type);
        if let Some(dn) = builder.dimension_names.as_ref()
            && dn.len() != shape.len()
        {
            return Err(format!(
                "Dimension names length {} does not match shape length {}",
                dn.len(),
                shape.len()
            ));
        }

        Ok(InlineArray {
            values: builder.values,
            data_type,
            shape,
            dimension_names: builder.dimension_names,
            attributes: builder.attributes,
        })
    }

    type Error = crate::Error;
}

impl ZarrConventionImpl for InlineArray {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid!("551e1edc-9717-4795-b149-5b855b00648c"),
        schema_url: uri!(
            "https://raw.githubusercontent.com/clbarnes/zarr-convention-inline-array/refs/tags/v1/schema.json"
        ),
        spec_url: uri!(
            "https://github.com/clbarnes/zarr-convention-inline-array/blob/v1/README.md"
        ),
        name: "inline_array",
        description: "Inline small array data in metadata",
    };
}

register_zarr_conventions!(InlineArray);
