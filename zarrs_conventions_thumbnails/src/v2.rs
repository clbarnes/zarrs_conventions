#![doc = include_str!("../README.md")]

use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use zarrs_conventions::iref::{UriRefBuf};
use zarrs_conventions::{
    ConventionDefinition, NestedRepr, ZarrConventionImpl, iref::uri, register_zarr_conventions,
    uuid::uuid,
};

/// A collection of thumbnails representing a Zarr node.
///
/// This is a thin wrapper around `Vec<Thumbnail>` that implements
/// the zarr convention traits. It derefs to `Vec<Thumbnail>`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Thumbnails(Vec<Thumbnail>);

impl Deref for Thumbnails {
    type Target = Vec<Thumbnail>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Thumbnails {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Thumbnail>> for Thumbnails {
    fn from(v: Vec<Thumbnail>) -> Self {
        Self(v)
    }
}

impl From<Thumbnails> for Vec<Thumbnail> {
    fn from(t: Thumbnails) -> Self {
        t.0
    }
}

impl FromIterator<Thumbnail> for Thumbnails {
    fn from_iter<I: IntoIterator<Item = Thumbnail>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl AsRef<[Thumbnail]> for Thumbnails {
    fn as_ref(&self) -> &[Thumbnail] {
        &self.0
    }
}

impl AsMut<[Thumbnail]> for Thumbnails {
    fn as_mut(&mut self) -> &mut [Thumbnail] {
        &mut self.0
    }
}

impl IntoIterator for Thumbnails {
    type Item = Thumbnail;
    type IntoIter = std::vec::IntoIter<Thumbnail>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Thumbnails {
    type Item = &'a Thumbnail;
    type IntoIter = std::slice::Iter<'a, Thumbnail>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Thumbnails {
    type Item = &'a mut Thumbnail;
    type IntoIter = std::slice::IterMut<'a, Thumbnail>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Thumbnails {
    /// Returns an iterator over the thumbnails.
    pub fn iter(&self) -> impl Iterator<Item = &Thumbnail> {
        self.0.iter()
    }

    /// Returns a mutable iterator over the thumbnails.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Thumbnail> {
        self.0.iter_mut()
    }
}

impl ZarrConventionImpl for Thumbnails {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid!("38a1d2ca-5f40-4ee2-b4d5-5e87bfeb7549"),
        schema_url: uri!(
            "https://raw.githubusercontent.com/zarr-conventions/thumbnails/refs/tags/v2/schema.json"
        ),
        spec_url: uri!("https://github.com/zarr-conventions/thumbnails/blob/v2/README.md"),
        name: "thumbnails",
        description: "Metadata for thumbnails representing Zarr data",
    };
}

impl NestedRepr for Thumbnails {
    const KEY: &'static str = "thumbnails";
}

register_zarr_conventions!(Thumbnails);

fn is_empty_map(map: &serde_json::Map<String, serde_json::Value>) -> bool {
    map.is_empty()
}

/// A single thumbnail representing a Zarr node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thumbnail {
    /// Thumbnail pixel width as a positive integer.
    width: NonZeroU32,
    /// Thumbnail pixel height as a positive integer.
    height: NonZeroU32,
    /// Media type (formerly MIME type) of the thumbnail.
    media_type: String,
    /// Free-text description of this thumbnail's context.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Unstructured arbitrary metadata about the thumbnail.
    #[serde(default, skip_serializing_if = "is_empty_map")]
    attributes: serde_json::Map<String, serde_json::Value>,
    /// Location (relative reference, absolute URL, or data URI).
    uri: UriRefBuf,
}

impl Thumbnail {
    /// Create a new thumbnail.
    ///
    /// Returns an error if `width` or `height` is zero, or if `media_type` is empty.
    ///
    /// ```
    /// use zarrs_conventions_thumbnails::{Thumbnail, ThumbnailLocation};
    ///
    /// let thumb = Thumbnail::try_new(
    ///     96,
    ///     96,
    ///     "image/jpeg",
    ///     ThumbnailLocation::new_path("thumbnails/thumb96.jpeg"),
    /// ).unwrap();
    /// ```
    pub fn try_new(
        width: u32,
        height: u32,
        media_type: impl Into<String>,
        uri: impl Into<String>,
    ) -> Result<Self, String> {
        let width =
            NonZeroU32::new(width).ok_or_else(|| "Thumbnail width must be positive".to_string())?;
        let height = NonZeroU32::new(height)
            .ok_or_else(|| "Thumbnail height must be positive".to_string())?;
        let media_type = media_type.into();
        if media_type.is_empty() {
            return Err("Thumbnail media_type must not be empty".to_string());
        }
        Ok(Self {
            width,
            height,
            media_type,
            description: None,
            attributes: serde_json::Map::new(),
            uri: UriRefBuf::from_string(uri.into()).map_err(|e| format!("Invalid URI: {e}"))?,
        })
    }

    /// Thumbnail pixel width.
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    /// Thumbnail pixel height.
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    /// Media type (formerly MIME type) of the thumbnail, e.g., "image/jpeg".
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Free-text description of this thumbnail's context; could be used as alt text.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the description.
    pub fn description_mut(&mut self) -> &mut Option<String> {
        &mut self.description
    }

    /// Unstructured arbitrary metadata about the thumbnail.
    pub fn attributes(&self) -> &serde_json::Map<String, serde_json::Value> {
        &self.attributes
    }

    /// Mutable access to the attributes map.
    pub fn attributes_mut(&mut self) -> &mut serde_json::Map<String, serde_json::Value> {
        &mut self.attributes
    }

    /// Thumbnail location, guaranteed to be a URI reference.
    pub fn uri(&self) -> &str {
        &self.uri
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zarrs_conventions::{
        AttributesBuilder, AttributesParser, ConventionId, DEFAULT_ZARR_CONVENTION_REGISTRY,
        ZarrConventionImpl,
    };

    use super::{Thumbnail, Thumbnails};

    #[test]
    fn is_registered() {
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY
                .contains(&ConventionId::Uuid(Thumbnails::DEFINITION.uuid))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SchemaUrl(
                Thumbnails::DEFINITION.schema_url.to_owned()
            ))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SpecUrl(
                Thumbnails::DEFINITION.spec_url.to_owned()
            ))
        );
    }

    #[test]
    fn pass_expected_with_path() {
        let value = json!({
            "zarr_conventions": [{"uuid": Thumbnails::DEFINITION.uuid}],
            "thumbnails": [
                {
                    "width": 96,
                    "height": 96,
                    "media_type": "image/jpeg",
                    "uri": "thumbnails/thumb96.jpeg"
                }
            ]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        let thumbnails: Thumbnails = parser.parse_nested().unwrap().unwrap();
        assert_eq!(thumbnails.len(), 1);
        assert_eq!(thumbnails[0].width().get(), 96);
        assert_eq!(thumbnails[0].uri(), "thumbnails/thumb96.jpeg");
    }

    #[test]
    fn pass_expected_with_url() {
        let value = json!({
            "zarr_conventions": [{"uuid": Thumbnails::DEFINITION.uuid}],
            "thumbnails": [
                {
                    "width": 48,
                    "height": 48,
                    "media_type": "image/png",
                    "uri": "https://image.host/thumb48.png"
                }
            ]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        let thumbnails: Thumbnails = parser.parse_nested().unwrap().unwrap();
        assert_eq!(thumbnails.len(), 1);
        assert_eq!(thumbnails[0].width().get(), 48);
    }

    #[test]
    fn fail_missing_location() {
        let value = json!({
            "zarr_conventions": [{"uuid": Thumbnails::DEFINITION.uuid}],
            "thumbnails": [
                {
                    "width": 96,
                    "height": 96,
                    "media_type": "image/jpeg"
                }
            ]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        assert!(parser.parse_nested::<Thumbnails>().is_err());
    }

    #[test]
    fn can_build_with_path() {
        let mut thumb =
            Thumbnail::try_new(96, 96, "image/jpeg", "thumbnails/thumb96.jpeg").unwrap();
        *thumb.description_mut() = Some("A test thumbnail".to_string());

        let thumbnails: Thumbnails = vec![thumb].into();
        let mut builder = AttributesBuilder::default();
        builder.add_nested(&thumbnails).unwrap();
        let attrs = builder.build().unwrap();
        println!("{attrs:#}");
    }

    #[test]
    fn can_build_with_url() {
        let thumb =
            Thumbnail::try_new(48, 48, "image/png", "https://image.host/thumb48.png").unwrap();

        let thumbnails: Thumbnails = vec![thumb].into();
        let mut builder = AttributesBuilder::default();
        builder.add_nested(&thumbnails).unwrap();
        let attrs = builder.build().unwrap();
        println!("{attrs:#}");
    }

    #[test]
    fn try_new_fails_with_empty_media_type() {
        assert!(Thumbnail::try_new(96, 96, "", "thumb.jpeg").is_err());
    }

    #[test]
    fn try_new_fails_with_zero_dimensions() {
        assert!(Thumbnail::try_new(0, 96, "image/jpeg", "thumb.jpeg").is_err());
        assert!(Thumbnail::try_new(96, 0, "image/jpeg", "thumb.jpeg").is_err());
    }

    #[test]
    fn attributes_default_empty_and_skip_serializing() {
        let thumb = Thumbnail::try_new(96, 96, "image/jpeg", "thumb.jpeg").unwrap();

        assert!(thumb.attributes().is_empty());

        // Serialize and check that attributes is not present
        let json = serde_json::to_value(&thumb).unwrap();
        assert!(!json.as_object().unwrap().contains_key("attributes"));
    }

    #[test]
    fn attributes_serialized_when_non_empty() {
        let mut thumb = Thumbnail::try_new(96, 96, "image/jpeg", "thumb.jpeg").unwrap();

        thumb
            .attributes_mut()
            .insert("z_slice".to_string(), serde_json::json!(123));

        let json = serde_json::to_value(&thumb).unwrap();
        assert!(json.as_object().unwrap().contains_key("attributes"));
        assert_eq!(json["attributes"]["z_slice"], 123);
    }
}
