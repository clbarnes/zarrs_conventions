# zarrs_conventions

An implementation of [zarr-conventions](https://github.com/zarr-conventions) for [zarrs](https://zarrs.dev/) ecosystem.

The core crate implementing the [zarr-conventions spec](https://github.com/zarr-conventions/zarr-conventions-spec) is [zarrs_conventions](./zarrs_conventions/).
There are also crates for

- [license](https://github.com/clbarnes/zarr-convention-license/): [zarrs_conventions_license](./zarrs_conventions_license/)
- [uom](https://github.com/clbarnes/zarr-convention-uom/): [zarrs_conventions_uom](./zarrs_conventions_uom/)

See the respective crate docs for usage examples.

## Contributing

The crates' integration tests depend on git submodules;
clone the project with `--recurse-submodules`
or use `git submodule update --recursive`.
