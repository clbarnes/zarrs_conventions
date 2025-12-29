#![doc = include_str!("../README.md")]
use std::collections::BTreeSet;

pub use iref;
use iref::UriBuf;
use serde::Deserialize;
pub use uuid;
use uuid::Uuid;

mod attributes;
pub use attributes::{AttributesBuilder, AttributesParser};
mod traits;
pub use traits::{FromNestedOrPrefixed, NestedRepr, PrefixedRepr, ZarrConventionImpl};

mod convention;
pub use convention::{Convention, ConventionDefinition};

pub mod registry;
pub use registry::DEFAULT_ZARR_CONVENTION_REGISTRY;

#[cfg(test)]
mod tests;

/// Unstructured user attributes map from a Zarr node.
pub type Attributes = serde_json::Map<String, serde_json::Value>;

/// Derived from the objects in the `zarr_conventions` attribute.
/// e.g.
///
/// ```jsonc
/// {
///   "uuid": "2dc8d146-3932-4e08-8542-06aa0e826508",
///   "schema_url": "https://raw.githubusercontent.com/zarr-experimental/proj-prefix/refs/tags/v1/schema.json",
///   "spec_url": "https://github.com/zarr-experimental/proj-prefix/blob/v1/README.md",
///   "name": "proj:",
///   "description": "Coordinate reference system information for geospatial data, using prefix namespacing."
/// }
/// ```
///
/// Only uuid, schema_url, and spec_url may be used to identify the convention, in that order of preference.
#[derive(Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
pub enum ConventionId {
    Uuid(Uuid),
    SchemaUrl(UriBuf),
    SpecUrl(UriBuf),
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
/// e.g. go from
///
/// ```json
/// {
///   "prefix:a": "a",
///   "prefix:one": 1,
///   "prefix:object": {"somekey": "somevalue"},
///   "otherfield": [],
/// }
/// ```
///
/// to
///
/// ```json
/// {
///   "a": "a",
///   "one": 1,
///   "object": {"somekey": "somevalue"},
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
