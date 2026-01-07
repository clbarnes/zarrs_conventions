#[doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, NestedRepr, ZarrConventionImpl, iref::uri, register_zarr_conventions,
    uuid::uuid,
};

/// Conventional metadata for units of measurement,
/// applied to numerical Zarr arrays.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UnitOfMeasurement {
    ucum: Ucum,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl UnitOfMeasurement {
    pub fn builder() -> Builder {
        Default::default()
    }

    pub fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }

    pub fn ucum(&self) -> &Ucum {
        &self.ucum
    }
}

/// Metadata using the [Unified Code for Units and Measures specification](https://ucum.org/ucum).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Ucum {
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
}

impl Ucum {
    /// **Case-sensitive** UCUM unit string,
    /// possibly including a magnitude term.
    ///
    /// If None, assume an arbitrary unit of magnitude 1.
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    /// Version of the UCUM specification, if defined.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

impl ZarrConventionImpl for UnitOfMeasurement {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid!("3bbe438d-df37-49fe-8e2b-739296d46dfb"),
        schema_url: uri!(
            "https://raw.githubusercontent.com/clbarnes/zarr-convention-uom/refs/tags/v1/schema.json"
        ),
        spec_url: uri!("https://github.com/clbarnes/zarr-convention-uom/blob/v1/README.md"),
        name: "uom",
        description: "Units of measurement for Zarr arrays",
    };
}

impl NestedRepr for UnitOfMeasurement {
    const KEY: &'static str = "uom";
}

register_zarr_conventions!(UnitOfMeasurement);

#[derive(Debug, Default)]
pub struct Builder {
    unit: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

impl Builder {
    /// Set the **case-sensitive** UCUM string,
    /// which may be a quantity (i.e. have a magnitude term).
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Set the UCUM specification version used here.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Describe the unit being measured in free text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Build the unit.
    pub fn build(self) -> UnitOfMeasurement {
        UnitOfMeasurement {
            ucum: Ucum {
                unit: self.unit,
                version: self.version,
            },
            description: self.description,
        }
    }
}

impl From<Builder> for UnitOfMeasurement {
    fn from(value: Builder) -> Self {
        value.build()
    }
}
