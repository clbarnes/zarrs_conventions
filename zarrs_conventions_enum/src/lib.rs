#![doc = include_str!("../README.md")]
use ordermap::OrderSet;
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, PrefixedRepr, ZarrConventionImpl, iref::uri, register_zarr_conventions,
    uuid,
};

/// Metadata for dictionary-encoded enum arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumMeta {
    ordered: bool,
    codes: OrderSet<String>,
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
    pub fn builder() -> EnumBuilder {
        Default::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.codes.iter().map(|s| s.as_str())
    }

    pub fn ordered(&self) -> bool {
        self.ordered
    }

    pub fn encode(&self, value: impl AsRef<str>) -> Option<usize> {
        let s = value.as_ref();
        self.codes.get_index_of(s)
    }

    pub fn decode(&self, index: usize) -> Option<&str> {
        self.codes.get_index(index).map(|s| s.as_str())
    }
}

impl From<EnumBuilder> for EnumMeta {
    fn from(builder: EnumBuilder) -> Self {
        builder.build()
    }
}

impl From<EnumMeta> for EnumBuilder {
    fn from(meta: EnumMeta) -> Self {
        Self {
            ordered: meta.ordered,
            codes: meta.codes,
        }
    }
}

/// Builder for [EnumMeta].
#[derive(Debug, Clone)]
pub struct EnumBuilder {
    ordered: bool,
    codes: OrderSet<String>,
}

impl Default for EnumBuilder {
    fn default() -> Self {
        Self {
            ordered: true,
            codes: Default::default(),
        }
    }
}

impl EnumBuilder {
    pub fn add_code(&mut self, code: impl Into<String>) -> &mut Self {
        let code = code.into();
        if self.ordered
            && let Some(last) = self.codes.last()
            && last > &code
        {
            self.ordered = false;
        }
        self.codes.insert(code);
        self
    }

    pub fn add_codes<I, C>(&mut self, codes: I) -> &mut Self
    where
        I: IntoIterator<Item = C>,
        C: Into<String>,
    {
        for code in codes {
            self.add_code(code);
        }
        self
    }

    pub fn sort(&mut self) -> &mut Self {
        if self.ordered {
            return self;
        }
        self.codes.sort_unstable();
        self.ordered = true;
        self
    }

    pub fn build(self) -> EnumMeta {
        EnumMeta {
            ordered: self.ordered,
            codes: self.codes,
        }
    }
}

impl PrefixedRepr for EnumMeta {
    const PREFIX: &'static str = "enum:";
}

impl<C: Into<String>> FromIterator<C> for EnumMeta {
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        let mut b = Self::builder();
        for code in iter {
            b.add_code(code);
        }
        b.build()
    }
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
        let mut ebuilder = EnumMeta::builder();
        ebuilder.add_code("a").add_code("b").add_code("c").sort();

        let enum_meta = ebuilder.build();
        let mut builder = AttributesBuilder::default();
        builder.add_prefixed(&enum_meta).unwrap();
        let attributes = builder.build().unwrap();
        let parser: AttributesParser = serde_json::from_value(attributes).unwrap();
        let _enum: EnumMeta = parser.parse_prefixed().unwrap().unwrap();
    }
}
