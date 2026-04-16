# zarrs_conventions_thumbnails

Rust implementation of the [Thumbnails Zarr convention](https://github.com/clbarnes/zarr-convention-thumbnails).

This convention allows a Zarr node to refer to thumbnails which represent that node in some way.

## Usage

```rust
use zarrs_conventions_thumbnails::{Thumbnails, Thumbnail, ThumbnailLocation};

// Create a thumbnail with a path
let mut thumb = Thumbnail::try_new(
    96,
    96,
    "image/jpeg",
    ThumbnailLocation::new_path("thumbnails/thumb96.jpeg"),
).unwrap();

// Set optional description
*thumb.description_mut() = Some("A 96x96 JPEG thumbnail".to_string());

// Add arbitrary attributes
thumb.attributes_mut().insert("z_slice".to_string(), serde_json::json!(123));

// Create thumbnails collection (Thumbnails derefs to Vec<Thumbnail>)
let thumbnails: Thumbnails = vec![thumb].into();
```
