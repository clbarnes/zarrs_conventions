use rstest::rstest;
use zarrs_conventions::{
    Convention, DEFAULT_ZARR_CONVENTION_REGISTRY, ZarrConventions, ZarrMetadata,
};

#[rstest]
fn test_examples(
    #[files("spec/examples/*.json")]
    #[mode = bytes]
    contents: &[u8],
) {
    let attrs = serde_json::from_slice::<ZarrMetadata>(contents)
        .expect("should be valid metadata")
        .attributes;
    let convention_data = attrs
        .get("zarr_conventions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for v in convention_data {
        let c: Convention = serde_json::from_value(v).expect("should be valid convention");
        let id = c.id();
        assert!(!DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&id));
    }

    let _conventions =
        ZarrConventions::from_attributes(&attrs).expect("should all be valid conventions");
}
