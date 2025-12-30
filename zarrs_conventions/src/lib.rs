#![doc = include_str!("../../README.md")]
use std::collections::BTreeSet;

/// Used for representing URLs.
pub use iref;
use iref::UriBuf;
use serde::{Serialize, Deserialize};
/// Used for uniquely identifying conventions.
pub use uuid;
use uuid::Uuid;

mod attributes;
pub use attributes::{AttributesBuilder, AttributesParser};
mod traits;
pub use traits::{NestedOrPrefixedRepr, NestedRepr, PrefixedRepr, ZarrConventionImpl};

mod convention;
pub use convention::{Convention, ConventionDefinition};

pub mod registry;
pub use registry::DEFAULT_ZARR_CONVENTION_REGISTRY;

#[cfg(test)]
mod tests;

/// Unstructured user attributes map from a Zarr node.
pub type Attributes = serde_json::Map<String, serde_json::Value>;

/// Identifier for a zarr convention.
///
/// Only uuid, schema_url, and spec_url may be used to identify the convention, in that order of preference.
#[derive(Debug, Clone, Serialize, Hash, PartialEq, Eq)]
pub enum ConventionId {
    Uuid(Uuid),
    SchemaUrl(UriBuf),
    SpecUrl(UriBuf),
}

impl From<ConventionDefinition> for ConventionId {
    fn from(value: ConventionDefinition) -> Self {
        value.id_uuid()
    }
}

impl From<Convention> for ConventionId {
    fn from(value: Convention) -> Self {
        if let Some(i) = value.uuid {
            Self::Uuid(i)
        } else if let Some(i) = value.schema_url {
            Self::SchemaUrl(i)
        } else if let Some(i) = value.spec_url {
            Self::SpecUrl(i)
        } else {
            unreachable!("one identifier must be defined")
        }
    }
}

/// Identifiers of zarr conventions in use for this node for efficient lookups.
///
/// ```jsonc
/// {
///   "node_type": "group",
///   "zarr_format": 3,
///   "attributes": {
///     "zarr_conventions": [...],  // derived from this field
///     ...
///   }
/// }
/// ```
#[derive(Debug, Default, Clone)]
pub struct ZarrConventions {
    uuids: BTreeSet<Uuid>,
    schema_urls: BTreeSet<UriBuf>,
    spec_urls: BTreeSet<UriBuf>,
}

impl ZarrConventions {
    const KEY: &'static str = "zarr_conventions";

    /// Get the set of in-use conventions from a zarr attributes map.
    pub fn from_attributes(attributes: &Attributes) -> serde_json::Result<Self> {
        let Some(zc) = attributes.get(Self::KEY) else {
            return Ok(ZarrConventions::default());
        };
        serde_json::from_value(zc.clone())
    }
}

impl<'de> Deserialize<'de> for ZarrConventions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let lst: Vec<Convention> = Deserialize::deserialize(deserializer)?;
        Ok(lst
            .into_iter()
            .fold(ZarrConventions::default(), |mut c, item| {
                if let Some(uuid) = item.uuid {
                    c.uuids.insert(uuid);
                }
                if let Some(schema_url) = item.schema_url {
                    c.schema_urls.insert(schema_url);
                }
                if let Some(spec_url) = item.spec_url {
                    c.spec_urls.insert(spec_url);
                }
                c
            }))
    }
}

impl From<Uuid> for ConventionId {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

/// Convert a flat prefixed representation into a nested representation.
///
/// e.g. go from
///
/// ```json
/// {
///   "prefix:a": "a",
///   "prefix:one": 1,
///   "prefix:object": {"somekey": "somevalue"},
///   "otherfield": []
/// }
/// ```
///
/// to
///
/// ```json
/// {
///   "a": "a",
///   "one": 1,
///   "object": {"somekey": "somevalue"}
/// }
/// ```
pub fn nest_prefixed(prefix: &str, map: &Attributes, out: Attributes) -> serde_json::Value {
    serde_json::Value::Object(
        map.iter()
            .filter_map(|(k, v)| Some((k.strip_prefix(prefix)?.to_string(), v.clone())))
            .fold(out, |mut acc, (k, v)| {
                acc.insert(k, v);
                acc
            }),
    )
}
