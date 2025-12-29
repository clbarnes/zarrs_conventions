use serde::{
    Serialize,
    de::{DeserializeOwned, Error},
};

use crate::{
    Attributes, ZarrConventions,
    convention::{Convention, ConventionDefinition},
    nest_prefixed,
};

/// Types should also implement at least one of [NestedRepr] and [PrefixedRepr].
pub trait ZarrConventionImpl {
    const DEFINITION: ConventionDefinition;

    fn in_use(identifiers: &ZarrConventions) -> bool {
        identifiers.uuids.contains(&Self::DEFINITION.uuid)
            || identifiers
                .schema_urls
                .contains(Self::DEFINITION.schema_url)
            || identifiers.spec_urls.contains(Self::DEFINITION.spec_url)
    }

    fn to_convention() -> Convention {
        Self::DEFINITION.into()
    }
}

/// Struct MUST serialize to a JSON Object (i.e. map with string keys).
pub trait PrefixedRepr: ZarrConventionImpl + DeserializeOwned + Serialize {
    /// Should include delimiter, e.g. `"proj:"`.
    const PREFIX: &'static str;

    fn from_attributes_prefixed(
        attributes: &serde_json::Map<String, serde_json::Value>,
    ) -> serde_json::Result<Self> {
        let nested = nest_prefixed(Self::PREFIX, attributes, Default::default());
        serde_json::from_value(nested)
    }

    fn to_attributes_prefixed(&self, output: &mut Attributes) -> serde_json::Result<()> {
        let value = serde_json::to_value(self)?;
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    output.insert(format!("{}{}", Self::PREFIX, k), v);
                }
                Ok(())
            }
            _ => Err(serde_json::Error::custom(
                "Prefixed representation must serialize to a JSON object",
            )),
        }
    }
}

pub trait NestedRepr: ZarrConventionImpl + DeserializeOwned + Serialize {
    const KEY: &'static str;

    fn from_attributes_nested(
        attributes: &serde_json::Map<String, serde_json::Value>,
    ) -> serde_json::Result<Self> {
        let cloned = attributes
            .get(Self::KEY)
            .ok_or_else(|| {
                serde_json::Error::custom(format!("Zarr convention key not found: '{}'", Self::KEY))
            })?
            .clone();
        serde_json::from_value(cloned)
    }

    fn to_attributes_nested(&self, output: &mut Attributes) -> serde_json::Result<()> {
        let value = serde_json::to_value(self)?;
        output.insert(Self::KEY.to_string(), value);
        Ok(())
    }
}

/// Try to deserialize either from nested or prefixed representation.
///
/// If the nested representation is found and the value of the top-level key is an object,
/// that object is combined with any prefixed keys to form the final object.
/// Otherwise, the value of the top-level key overrides prefixed keys (all prefixed keys are ignored).
pub trait FromNestedOrPrefixed: NestedRepr + PrefixedRepr {
    fn from_attributes(
        attributes: &serde_json::Map<String, serde_json::Value>,
    ) -> serde_json::Result<Self> {
        if let Some(cloned) = attributes.get(Self::KEY).cloned() {
            if let serde_json::Value::Object(m) = cloned {
                serde_json::from_value(nest_prefixed(Self::PREFIX, attributes, m))
            } else {
                serde_json::from_value(cloned)
            }
        } else {
            Self::from_attributes_prefixed(attributes)
        }
    }
}

impl<T: NestedRepr + PrefixedRepr> FromNestedOrPrefixed for T {}

#[cfg(test)]
mod tests {
    use ctor::ctor;
    use iref::uri;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    use crate::{
        Attributes, FromNestedOrPrefixed, NestedRepr, PrefixedRepr, ZarrConventionImpl,
        ZarrConventions, convention::ConventionDefinition,
    };

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Proj {
        code: String,
    }

    impl ZarrConventionImpl for Proj {
        const DEFINITION: ConventionDefinition = ConventionDefinition {
            uuid: uuid::uuid!("ef154843-db6c-41c3-8ccf-64294a8fa889"),
            schema_url: uri!(
                "https://raw.githubusercontent.com/zarr-experimental/proj-nested-key/refs/tags/v1/schema.json"
            ),
            spec_url: uri!("https://example.com/specs/proj"),
            name: "proj",
            description: "Coordinate reference system information for geospatial data, using keyed namespacing.",
        };
    }

    impl PrefixedRepr for Proj {
        const PREFIX: &'static str = "proj:";
    }

    impl NestedRepr for Proj {
        const KEY: &'static str = "proj";
    }

    fn make_zarr_conventions() -> serde_json::Value {
        json!([
            {
                "uuid": "ef154843-db6c-41c3-8ccf-64294a8fa889",
                "schema_url": "https://raw.githubusercontent.com/zarr-experimental/proj-nested-key/refs/tags/v1/schema.json",
                "spec_url": "https://example.com/specs/proj",
                "name": "proj",
                "description": "Coordinate reference system information for geospatial data, using keyed namespacing."
            }
        ])
    }

    fn make_flat() -> serde_json::Value {
        json!({
                "zarr_conventions": make_zarr_conventions(),
                "proj:code": "EPSG:4326"
        })
    }

    fn make_nested() -> serde_json::Value {
        json!({
                "zarr_conventions": make_zarr_conventions(),
                "proj": {
                    "code": "EPSG:4326"
                }
        })
    }

    fn make_expected() -> Proj {
        Proj {
            code: "EPSG:4326".to_string(),
        }
    }

    fn into_object(value: serde_json::Value) -> Attributes {
        match value {
            serde_json::Value::Object(m) => m,
            _ => panic!("Expected JSON object"),
        }
    }

    #[test]
    fn from_attributes_nested() {
        let attrs: Attributes = into_object(make_nested());
        let conventions = ZarrConventions::from_attributes(&attrs).unwrap();
        assert!(Proj::in_use(&conventions));
        let expected = make_expected();
        let proj = Proj::from_attributes_nested(&attrs).unwrap();
        assert_eq!(proj, expected);
    }

    #[test]
    fn from_attributes_prefixed() {
        let attrs: Attributes = into_object(make_flat());
        let conventions = ZarrConventions::from_attributes(&attrs).unwrap();
        assert!(Proj::in_use(&conventions));
        let expected = make_expected();
        let proj = Proj::from_attributes_prefixed(&attrs).unwrap();
        assert_eq!(proj, expected);
    }

    #[test]
    fn from_attributes_nested_or_prefixed() {
        let flat: Attributes = into_object(make_flat());
        let nested: Attributes = into_object(make_nested());
        let proj_from_nested = Proj::from_attributes(&nested).unwrap();
        let proj_from_flat = Proj::from_attributes(&flat).unwrap();
        assert_eq!(proj_from_nested, proj_from_flat);
    }

    #[ctor]
    fn register_proj() {
        crate::DEFAULT_ZARR_CONVENTION_REGISTRY.register::<Proj>();
    }

    #[test]
    fn proj_registered() {
        let registry = &crate::DEFAULT_ZARR_CONVENTION_REGISTRY;
        let id = crate::ConventionId::Uuid(Proj::DEFINITION.uuid);
        assert!(registry.contains(&id));
        let convention = registry.get(&id).expect("Convention not found");
        assert_eq!(convention.name, Proj::DEFINITION.name);
    }
}
