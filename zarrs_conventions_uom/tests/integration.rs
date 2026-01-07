use rstest::rstest;
use zarrs_conventions::{
    DEFAULT_ZARR_CONVENTION_REGISTRY, NestedRepr, ZarrConventionImpl, ZarrConventions, ZarrMetadata,
};
use zarrs_conventions_uom::UnitOfMeasurement;

#[test]
fn is_registered() {
    assert!(DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&UnitOfMeasurement::DEFINITION.id_uuid()));
}

#[rstest]
fn test_examples(
    #[files("spec/examples/*.json")]
    #[mode = bytes]
    contents: &[u8],
) {
    let attrs = serde_json::from_slice::<ZarrMetadata>(contents)
        .expect("should be valid metadata")
        .attributes;

    let conventions =
        ZarrConventions::from_attributes(&attrs).expect("should all be valid conventions");
    assert!(conventions.contains(&UnitOfMeasurement::DEFINITION.id_uuid()));

    let _uom = UnitOfMeasurement::from_attributes_nested(&attrs).expect("should be present");
}
