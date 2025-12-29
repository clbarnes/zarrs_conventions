use iref::uri;

use crate::{NestedRepr, PrefixedRepr, ZarrConventionImpl, convention::ConventionDefinition};

#[allow(unused)]
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct MustBeNested {
    pub a: u8,
    pub b: u8,
}

impl ZarrConventionImpl for MustBeNested {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("11111111-1111-1111-1111-111111111111"),
        schema_url: uri!("https://example.com/schemas/must_be_nested.json"),
        spec_url: uri!("https://example.com/specs/must_be_nested"),
        name: "must_be_nested",
        description: "A convention that must be represented in nested form.",
    };
}

impl NestedRepr for MustBeNested {
    const KEY: &'static str = "must_be_nested";
}

#[allow(unused)]
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct MustBePrefixed {
    pub x: u8,
    pub y: u8,
}

impl ZarrConventionImpl for MustBePrefixed {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("22222222-2222-2222-2222-222222222222"),
        schema_url: uri!("https://example.com/schemas/mustprefixed.json"),
        spec_url: uri!("https://example.com/specs/mustprefixed"),
        name: "must_be_prefixed",
        description: "A convention that must be represented in prefixed form.",
    };
}

impl PrefixedRepr for MustBePrefixed {
    const PREFIX: &'static str = "must_be_prefixed:";
}

#[allow(unused)]
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct CanBeEither {
    pub foo: u8,
    pub bar: u8,
}

impl ZarrConventionImpl for CanBeEither {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("33333333-3333-3333-3333-333333333333"),
        schema_url: uri!("https://example.com/schemas/can_be_either.json"),
        spec_url: uri!("https://example.com/specs/can_be_either"),
        name: "can_be_either",
        description: "A convention that can be represented in either nested or prefixed form.",
    };
}

impl NestedRepr for CanBeEither {
    const KEY: &'static str = "can_be_either";
}

impl PrefixedRepr for CanBeEither {
    const PREFIX: &'static str = "can_be_either:";
}
