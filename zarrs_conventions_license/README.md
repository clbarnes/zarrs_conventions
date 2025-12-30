# zarrs_convention_license

The [license](https://github.com/clbarnes/zarr-convention-license/) [zarr convention](https://github.com/zarr-conventions/) for the [zarrs](https://zarrs.dev) ecosystem.

For use with the `zarrs_conventions` crate.

## Usage

```rust
use zarrs_conventions_license::{
    License,
    LicenseItem,
};

let license = License::from_iter([
    LicenseItem::new_spdx("MIT"),
    LicenseItem::new_url("https://www.apache.org/licenses/LICENSE-2.0.txt".parse().unwrap()),
]);

let spdx = license.as_ref()[0].spdx().unwrap();
```
