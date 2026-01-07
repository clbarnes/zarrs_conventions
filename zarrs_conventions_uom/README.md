# zarrs_convention_uom

The [uom](https://github.com/clbarnes/zarr-convention-uom/) [zarr convention](https://github.com/zarr-conventions/) for the [zarrs](https://zarrs.dev) ecosystem.

For use with the `zarrs_conventions` crate.

## Usage

```rust
use zarrs_conventions_uom::UnitOfMeasurement;

let uom = UnitOfMeasurement::builder()
    .unit("kg")
    .description("how heavy my apples are")
    .build();
```
