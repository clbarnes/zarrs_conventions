use std::collections::BTreeSet;

use serde::Deserialize;

use crate::{
    Attributes, NestedOrPrefixedRepr, NestedRepr, PrefixedRepr, ZarrConventionImpl,
    ZarrConventions,
    convention::{ConventionBuilder, ConventionDefinition},
};

/// Type for building zarr attributes,
/// including conventional and unstructured metadata.
#[derive(Debug, Clone)]
pub struct AttributesBuilder {
    convention_definitions: BTreeSet<ConventionDefinition>,
    attributes: Attributes,
    uuid: bool,
    schema_url: bool,
    spec_url: bool,
    name: bool,
    description: bool,
}

impl Default for AttributesBuilder {
    fn default() -> Self {
        Self {
            convention_definitions: BTreeSet::default(),
            attributes: Attributes::default(),
            uuid: true,
            schema_url: true,
            spec_url: true,
            name: true,
            description: true,
        }
    }
}

impl AttributesBuilder {
    /// Whether to include the conventions' UUID.
    pub fn uuid(&mut self, enable: bool) -> &mut Self {
        self.uuid = enable;
        self
    }

    /// Whether to include the conventions' schema URL.
    pub fn schema_url(&mut self, enable: bool) -> &mut Self {
        self.schema_url = enable;
        self
    }

    /// Whether to include the conventions' specification URL.
    pub fn spec_url(&mut self, enable: bool) -> &mut Self {
        self.spec_url = enable;
        self
    }

    /// Whether to include the conventions' name.
    pub fn name(&mut self, enable: bool) -> &mut Self {
        self.name = enable;
        self
    }

    /// Whether to include the conventions' description.
    pub fn description(&mut self, enable: bool) -> &mut Self {
        self.description = enable;
        self
    }

    /// Often not necessary, as other methods will add the convention automatically.
    fn add_convention<T: crate::ZarrConventionImpl>(&mut self) -> &mut Self {
        self.convention_definitions.insert(T::DEFINITION);
        self
    }

    /// Add conventional metadata in nested form.
    /// Also adds the convention to the list of in-use conventions.
    pub fn add_nested<T: NestedRepr>(&mut self, value: &T) -> serde_json::Result<&mut Self> {
        value.to_attributes_nested(&mut self.attributes)?;
        self.add_convention::<T>();
        Ok(self)
    }

    /// Add conventional metadata in prefixed form.
    /// Also adds the convention to the list of in-use conventions.
    pub fn add_prefixed<T: PrefixedRepr>(&mut self, value: &T) -> serde_json::Result<&mut Self> {
        value.to_attributes_prefixed(&mut self.attributes)?;
        self.add_convention::<T>();
        Ok(self)
    }

    /// Add an arbitrary attribute.
    pub fn add_attribute(
        &mut self,
        key: impl Into<String>,
        value: impl serde::Serialize,
    ) -> serde_json::Result<&mut Self> {
        let value = serde_json::to_value(value)?;
        self.attributes.insert(key.into(), value);
        Ok(self)
    }

    /// Build the final attributes map.
    pub fn build(mut self) -> serde_json::Result<serde_json::Value> {
        if !self.uuid
            && !self.schema_url
            && !self.spec_url
            && !self.convention_definitions.is_empty()
        {
            // No convention identifiers selected, so skip adding the conventions attribute.
            return Err(serde::ser::Error::custom(
                "At least one convention identifier (uuid, schema_url, spec_url) must be enabled",
            ));
        }

        if !self.convention_definitions.is_empty() {
            let res: serde_json::Result<Vec<serde_json::Value>> = self
                .convention_definitions
                .into_iter()
                .map(|d| {
                    let mut cb = ConventionBuilder::default();
                    if self.uuid {
                        cb = cb.uuid(d.uuid);
                    }
                    if self.schema_url {
                        cb = cb.schema_url(d.schema_url.to_owned());
                    }
                    if self.spec_url {
                        cb = cb.spec_url(d.spec_url.to_owned());
                    }
                    if self.name {
                        cb = cb.name(d.name);
                    }
                    if self.description {
                        cb = cb.description(d.description);
                    }
                    let c = cb.build().expect("convention definition should build");
                    serde_json::to_value(c)
                })
                .collect();
            let conventions = res?;

            self.attributes.insert(
                ZarrConventions::KEY.to_string(),
                serde_json::Value::Array(conventions),
            );
        }

        Ok(serde_json::Value::Object(self.attributes))
    }
}

/// Retrieve conventional and unstructured metadata from an attributes map.
#[derive(Debug, Clone, Deserialize)]
pub struct AttributesParser {
    #[serde(default)]
    zarr_conventions: ZarrConventions,
    #[serde(flatten)]
    fields: Attributes,
}

impl AttributesParser {
    /// Check whether a particular convention is in use.
    pub fn in_use<T: ZarrConventionImpl>(&self) -> bool {
        T::in_use(&self.zarr_conventions)
    }

    /// Parse conventional metadata from a nested representation, if supported.
    ///
    /// None if the convention is not listed in "zarr_conventions".
    pub fn parse_nested<T: NestedRepr>(&self) -> serde_json::Result<Option<T>> {
        if !T::in_use(&self.zarr_conventions) {
            return Ok(None);
        }
        T::from_attributes_nested(&self.fields).map(Some)
    }

    /// Parse conventional metadata from a prefixed representation, if supported.
    ///
    /// None if the convention is not listed in "zarr_conventions".
    pub fn parse_prefixed<T: PrefixedRepr>(&self) -> serde_json::Result<Option<T>> {
        if !T::in_use(&self.zarr_conventions) {
            return Ok(None);
        }
        T::from_attributes_prefixed(&self.fields).map(Some)
    }

    /// Parse conventional data from either a nested or prefixed representation,
    /// or a mixture, if both are supported.
    ///
    /// None if the convention is not listed in "zarr_conventions".
    pub fn parse<T: NestedOrPrefixedRepr>(&self) -> serde_json::Result<Option<T>> {
        if !T::in_use(&self.zarr_conventions) {
            return Ok(None);
        }
        T::from_attributes(&self.fields).map(Some)
    }

    /// Get an unstructured attribute.
    ///
    /// None if not present.
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> serde_json::Result<Option<T>> {
        let Some(v) = self.fields.get(key).cloned() else {
            return Ok(None);
        };
        serde_json::from_value(v).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AttributesBuilder,
        tests::{CanBeEither, MustBeNested, MustBePrefixed},
    };

    fn example() -> serde_json::Value {
        serde_json::json!({
            "zarr_conventions": [
                {
                    "uuid": "11111111-1111-1111-1111-111111111111",
                    "schema_url": "https://example.com/schemas/must_be_nested.json",
                    "spec_url": "https://example.com/specs/must_be_nested",
                    "name": "must_be_nested",
                    "description": "A convention that must be represented in nested form."
                },
                {
                    "uuid": "22222222-2222-2222-2222-222222222222",
                    "schema_url": "https://example.com/schemas/must_be_prefixed.json",
                    "spec_url": "https://example.com/specs/must_be_prefixed",
                    "name": "must_be_prefixed",
                    "description": "A convention that must be represented in prefixed form."
                },
                {
                    "uuid": "33333333-3333-3333-3333-333333333333",
                    "schema_url": "https://example.com/schemas/can_be_either.json",
                    "spec_url": "https://example.com/specs/can_be_either",
                    "name": "can_be_either",
                    "description": "A convention that can be represented in either nested or prefixed form."
                }
            ],
            "must_be_nested": {
                "a": 1,
                "b": 2
            },
            "must_be_prefixed:x": 3,
            "must_be_prefixed:y": 4,
            "can_be_either": {
                "foo": 5,
            },
            "can_be_either:bar": 6,
            "other_key": "other_value"
        })
    }

    #[test]
    fn test_attributes_parser_all() {
        let val = example();
        let parser: super::AttributesParser = serde_json::from_value(val).unwrap();

        let nest: MustBeNested = parser.parse_nested().unwrap().unwrap();
        assert_eq!(nest, MustBeNested { a: 1, b: 2 });

        let pref: MustBePrefixed = parser.parse_prefixed().unwrap().unwrap();
        assert_eq!(pref, MustBePrefixed { x: 3, y: 4 });

        let either: CanBeEither = parser.parse().unwrap().unwrap();
        assert_eq!(either, CanBeEither { foo: 5, bar: 6 });

        let other: String = parser.get("other_key").unwrap().unwrap();
        assert_eq!(other, "other_value");
    }

    #[test]
    fn test_attributes_builder() {
        let mut builder = AttributesBuilder::default();
        builder.add_nested(&MustBeNested { a: 1, b: 2 }).unwrap();
        builder
            .add_prefixed(&MustBePrefixed { x: 3, y: 4 })
            .unwrap();
        builder.add_attribute("other_key", "other_value").unwrap();
        builder
            .add_prefixed(&CanBeEither { foo: 5, bar: 6 })
            .unwrap();
        let val = builder.build().unwrap();

        let parser: super::AttributesParser = serde_json::from_value(val).unwrap();
        parser.in_use::<MustBeNested>();
        parser.in_use::<MustBePrefixed>();
        parser.in_use::<CanBeEither>();

        let nest: MustBeNested = parser.parse_nested().unwrap().unwrap();
        assert_eq!(nest, MustBeNested { a: 1, b: 2 });

        let pref: MustBePrefixed = parser.parse_prefixed().unwrap().unwrap();
        assert_eq!(pref, MustBePrefixed { x: 3, y: 4 });

        let either: CanBeEither = parser.parse().unwrap().unwrap();
        assert_eq!(either, CanBeEither { foo: 5, bar: 6 });

        let other: String = parser.get("other_key").unwrap().unwrap();
        assert_eq!(other, "other_value");
    }
}
