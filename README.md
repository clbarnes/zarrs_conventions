# zarrs_conventions

An implementation of [zarr-conventions](https://github.com/zarr-conventions) for [zarrs](https://zarrs.dev/) ecosystem.

## Usage

### Defining a convention convention

```rust
use zarrs_conventions::{
    NestedRepr, PrefixedRepr, ZarrConventionImpl, ZarrConventions, ConventionDefinition, register_zarr_conventions,
};
// re-exported crates
use zarrs_conventions::{uuid, iref};

/// Example from the conventions spec:
/// <https://github.com/zarr-conventions/zarr-conventions-spec>
#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
struct Proj {
    code: String,
}

impl ZarrConventionImpl for Proj {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("ef154843-db6c-41c3-8ccf-64294a8fa889"),
        schema_url: iref::uri!(
            "https://raw.githubusercontent.com/zarr-experimental/proj-nested-key/refs/tags/v1/schema.json"
        ),
        spec_url: iref::uri!("https://example.com/specs/proj"),
        name: "proj",
        description: "Coordinate reference system information for geospatial data.",
    };
}

// Allows the type to be represented in `{ ..., "attributes": { ..., "proj:code": "mycode" } }` form.
// Optional (so long as NestedRepr is implemented).
impl PrefixedRepr for Proj {
    const PREFIX: &'static str = "proj:";
}

// Allows the type to be represented in `{ ..., "attributes": { ..., "proj": { "code": "mycode" } } }` form.
// Optional (so long as PrefixedRepr is implemented).
impl NestedRepr for Proj {
    const KEY: &'static str = "proj";
}

// Allow this convention to be discovered at runtime by importers of this module.
// Not strictly necessary.
register_zarr_conventions!(Proj);
```

### Working with conventional metadata

```rust,ignore
use serde_json::{Value, json};
use zarr_conventions::{AttributesParser, AttributesBuilder};

// From the `"attributes"` field of Zarr v3 metadata.
// The full metadata document may look like
// `{ "zarr_format": 3, "node_type": "group", "attributes": ... }`
let attributes = json!(
    {
        "zarr_conventions": [
            {"schema_url": "https://example.com/schema/nested.json"},
            {"schema_url": "https://example.com/schema/prefixed.json"},
            {"schema_url": "https://example.com/schema/either.json"},
            {"schema_url": "https://example.com/schema/something_else.json"},
        ],
        "nested": {"bouba": "kiki"},
        "prefixed:foo": "bar",
        "either": {"alice": "bob"},
        "either:charlie": "dan",
        "other_key": "other_value",
    }
);

// Assume these types are fully defined and have the appropriate *Repr traits implemented.
struct Nested;
struct Prefixed;
struct Either;
// We don't need to have defined the final "something_else" convention; the parser will just treat it as unstructured.

let parser: AttributesParser = serde_json::from_value(attributes);

// Ok(None) if the requested convention is not listed in zarr_conventions.
// Err if it is, but cannot be deserialised.
let maybe_nested: Option<Nested> = parser.parse_nested().unwrap();
let maybe_prefixed: Option<Prefixed> = parser.parse_prefixed().unwrap();
let maybe_either: Option<Either> = parser.parse().unwrap();

// Unstructured attributes can still be retrieved.
let other_value: Option<String> = parser.get("other_key").unwrap();

// Prepare to write metadata
let builder = AttributesBuilder::default();
// Disable writing of the name and description in zarr_conventions,
// to keep the length down.
// At least one of uuid, spec_url, schema_url must remain true.
builder.name(false).description(false);

// The "zarr_conventions" field will be populated automatically
// when conventional metadata is added.
builder.add_nested(maybe_nested.unwrap()).unwrap();
builder.add_prefixed(maybe_prefixed.unwrap()).unwrap();
// For conventional metadata which can be represented either way,
// you have to pick one when deserialising.
builder.add_nested(maybe_either.unwrap()).unwrap();

// You can add arbitrary attributes
builder.add_attribute("other_key", other_value).unwrap();

let value = builder.build().unwrap();
println!("{value:#}");
```
