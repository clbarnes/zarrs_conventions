#![doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, PrefixedRepr, ZarrConventionImpl, iref::uri, register_zarr_conventions,
    uuid,
};

/// Metadata for dictionary-encoded enum arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumMeta {
    /// Whether the codes in the `enum_values` array are in sorted order.
    ordered: bool,
    /// Paths to zarr arrays of integer codes,
    /// indexing into a values array.
    pub codes: Vec<String>,
}

impl ZarrConventionImpl for EnumMeta {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("c906c423-56ed-413c-9943-7b2ff52d18f2"),
        schema_url: uri!(
            "https://raw.githubusercontent.com/DOES_NOT_EXIST/zarr-convention-categorical/refs/tags/v1/schema.json"
        ),
        spec_url: uri!(
            "https://github.com/DOES_NOT_EXIST/zarr-convention-categorical/blob/v1/README.md"
        ),
        name: "enum",
        description: "Dictionary-encoded enum array: integer codes indexing into a values array.",
    };
}

impl EnumMeta {
    pub fn new(ordered: bool, codes: Vec<String>) -> Self {
        Self { ordered, codes }
    }

    pub fn from_values<T: PartialOrd + Clone>(
        codes: Vec<String>,
        values: impl IntoIterator<Item = T>,
    ) -> Self {
        let mut last_val = None;
        for val in values.into_iter() {
            if last_val.is_some_and(|last| val < last) {
                return Self {
                    ordered: false,
                    codes,
                };
            }
            last_val = Some(val);
        }
        Self {
            ordered: true,
            codes,
        }
    }

    pub fn ordered(&self) -> bool {
        self.ordered
    }
}

impl PrefixedRepr for EnumMeta {
    const PREFIX: &'static str = "enum:";
}

register_zarr_conventions!(EnumMeta);

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zarrs_conventions::{
        AttributesParser, ConventionId, DEFAULT_ZARR_CONVENTION_REGISTRY, ZarrConventionImpl,
    };

    use crate::EnumMeta;

    #[test]
    fn is_registered() {
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY
                .contains(&ConventionId::Uuid(EnumMeta::DEFINITION.uuid))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SchemaUrl(
                EnumMeta::DEFINITION.schema_url.to_owned()
            ))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SpecUrl(
                EnumMeta::DEFINITION.spec_url.to_owned()
            ))
        );
    }

    #[test]
    fn pass_expected() {
        let value = json!({
            "zarr_conventions": [{"uuid": EnumMeta::DEFINITION.uuid}],
            "enum:ordered": true,
            "enum:codes": ["a", "b", "c"]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        let _meta: EnumMeta = parser.parse_prefixed().unwrap().unwrap();
    }

    #[test]
    fn fail_empty() {
        let value = json!({
            "zarr_conventions": [{"uuid": EnumMeta::DEFINITION.uuid}],
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        assert!(parser.parse_prefixed::<EnumMeta>().is_err());
    }
}
