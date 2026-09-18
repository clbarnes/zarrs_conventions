# zarrs_conventions_enum

The [enum](https://github.com/ilan-gold/zarr-convention-enum) [zarr convention](https://github.com/zarr-conventions/) for the [zarrs](https://zarrs.dev) ecosystem.

For use with the `zarrs_conventions` crate.

## Usage

```rust
use zarrs_conventions_enum::{EnumMeta, EnumEncoder, EnumDecoder};

let meta = EnumMeta {
    ordered: true,
    codes: vec!["a".into(), "b".into(), "c".into()]
};

let mut encoder: EnumEncoder<&'static str> = EnumEncoder::default();
assert!(encoder.encode("potato").is_none());
let (index, is_new) = encoder.insert("potato");
assert_eq!(index, 0);
assert!(is_new);

let (index2, is_new2) = encoder.insert("potato");
assert_eq!(index2, 0);
assert!(!is_new2);

let maybe_index3 = encoder.encode("potato");
assert_eq!(maybe_index3, Some(0));

let decoder = EnumDecoder::from(encoder);
let name = decoder.decode(0);
assert_eq!(name, Some("potato"));

let not_name = decoder.decode(999);
assert!(name.is_none());
```
