# zarrs_convention_license

The [license](https://github.com/clbarnes/zarr-convention-license/) [zarr convention](https://github.com/zarr-conventions/) for the [zarrs](https://zarrs.dev) ecosystem.

For use with the `zarrs_conventions` crate.

## Usage

```rust
use zarrs_conventions_license::License;

let license = License::new_spdx("MIT");
let spdx = license.spdx().unwrap();
```
