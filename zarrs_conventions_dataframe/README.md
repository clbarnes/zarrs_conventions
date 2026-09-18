# zarrs_conventions_enum

The [dataframe](https://github.com/ilan-gold/zarr-convention-dataframe) [zarr convention](https://github.com/zarr-conventions/) for the [zarrs](https://zarrs.dev) ecosystem.

For use with the `zarrs_conventions` crate.

## Usage

```rust
use zarrs_conventions_dataframe::DataFrameMeta;

let meta = DataFrameMeta::new(vec!["a".into(), "b".into(), "c".into()]);

// Constructing with an index is fallible as the index may be out of bounds
let with_index = DataFrameMeta::new_with_index(vec!["i".into(), "a".into(), "b".into(), "c".into()], 0).unwrap();
```
