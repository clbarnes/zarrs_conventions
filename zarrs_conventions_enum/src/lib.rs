#![doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, PrefixedRepr, ZarrConventionImpl, iref::uri, register_zarr_conventions,
    uuid,
};
mod lookup;
pub use lookup::{EnumDecoder, EnumEncoder};

/// Single license applicable to the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumMeta {
    pub ordered: bool,
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

impl PrefixedRepr for EnumMeta {
    const PREFIX: &'static str = "enum:";
}

register_zarr_conventions!(EnumMeta);

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zarrs_conventions::{
        AttributesBuilder, AttributesParser, ConventionId, DEFAULT_ZARR_CONVENTION_REGISTRY,
        ZarrConventionImpl,
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
        let _license: EnumMeta = parser.parse_prefixed().unwrap().unwrap();
    }

    #[test]
    fn fail_empty() {
        let value = json!({
            "zarr_conventions": [{"uuid": EnumMeta::DEFINITION.uuid}],
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        assert!(parser.parse_prefixed::<EnumMeta>().is_err());
    }

    #[test]
    fn can_build() {
        let enum_meta = EnumMeta {
            ordered: true,
            codes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        let mut builder = AttributesBuilder::default();
        builder.add_prefixed(&enum_meta).unwrap();
        let attributes = builder.build().unwrap();
        let parser: AttributesParser = serde_json::from_value(attributes).unwrap();
        let _license: EnumMeta = parser.parse_prefixed().unwrap().unwrap();
    }
}
