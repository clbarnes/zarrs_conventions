#![doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, NestedRepr, ZarrConventionImpl,
    iref::{Uri, UriBuf, uri},
    register_zarr_conventions, uuid,
};

/// Single license applicable to the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Inner", into = "Inner")]
pub struct License(Inner);

impl From<License> for Inner {
    fn from(value: License) -> Self {
        value.0
    }
}

impl TryFrom<Inner> for License {
    type Error = String;

    fn try_from(value: Inner) -> Result<Self, Self::Error> {
        if value.spdx.is_none()
            && value.url.is_none()
            && value.text.is_none()
            && value.file.is_none()
            && value.path.is_none()
        {
            return Err("At least one field must be set for LicenseItem".to_string());
        }
        Ok(License(value))
    }
}

/// Inner type used by the [License] and [Builder] types.
/// May contain incomplete or invalid data (i.e. no identifier).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Inner {
    #[serde(skip_serializing_if = "Option::is_none")]
    spdx: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<UriBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

impl License {
    /// Builder for constructing a [License].
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Create a new license item from an SPDX identifier.
    pub fn new_spdx<S: Into<String>>(identifier: S) -> Self {
        Self(Inner {
            spdx: Some(identifier.into()),
            ..Default::default()
        })
    }

    /// License as an SPDX identifier.
    /// Should not be a multi-license expression.
    pub fn spdx(&self) -> Option<&str> {
        self.0.spdx.as_deref()
    }

    /// Create a new license item from a URL to the license text.
    pub fn new_url(url: UriBuf) -> Self {
        Self(Inner {
            url: Some(url),
            ..Default::default()
        })
    }

    /// URL to the full license text.
    pub fn url(&self) -> Option<&Uri> {
        self.0.url.as_deref()
    }

    /// Create a new license item from a URL to the license text.
    pub fn new_text<S: Into<String>>(text: S) -> Self {
        Self(Inner {
            text: Some(text.into()),
            ..Default::default()
        })
    }

    /// Full license text.
    pub fn text(&self) -> Option<&str> {
        self.0.text.as_deref()
    }

    /// Create a new license item from a relative path to an object containing the license text.
    pub fn new_file<S: Into<String>>(file: S) -> Self {
        Self(Inner {
            file: Some(file.into()),
            ..Default::default()
        })
    }

    /// Relative path to an object containing the full license text.
    pub fn file(&self) -> Option<&str> {
        self.0.file.as_deref()
    }

    /// Create a new license item from a relative path to a zarr node with license metadata.
    pub fn new_path<S: Into<String>>(path: S) -> Self {
        Self(Inner {
            path: Some(path.into()),
            ..Default::default()
        })
    }

    /// Relative path to a zarr node with license metadata which also applies to this node.
    pub fn path(&self) -> Option<&str> {
        self.0.path.as_deref()
    }
}

impl ZarrConventionImpl for License {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("b77365e5-2b0c-4141-b917-c03b7c68e935"),
        schema_url: uri!(
            "https://raw.githubusercontent.com/clbarnes/zarr-convention-license/refs/tags/v1/schema.json"
        ),
        spec_url: uri!("https://github.com/clbarnes/zarr-convention-license/blob/v1/README.md"),
        name: "license",
        description: "Dataset licensing information.",
    };
}

impl NestedRepr for License {
    const KEY: &'static str = "license";
}

register_zarr_conventions!(License);

/// Builder for [License]s, created by [License::builder].
///
/// At least one license identifier must be set.
/// It is recommended to only set one.
/// In order of preference, `spdx > url > text > file > path`.
///
/// ```
/// use zarrs_conventions_license::License;
///
/// let item = License::builder()
///     .spdx("MIT")
///     .url("https://opensource.org/license/mit".parse().unwrap())
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Builder {
    inner: Inner,
    short: bool,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            inner: Inner {
                spdx: None,
                url: None,
                text: None,
                file: None,
                path: None,
            },
            short: false,
        }
    }
}

impl Builder {
    /// Shorten the license metadata by only keeping the most preferred form.
    pub fn short(mut self, short: bool) -> Self {
        self.short = short;
        self
    }

    /// SPDX license identifier; preferred over all.
    ///
    /// Should not be a multi-license expression.
    pub fn spdx(mut self, spdx: impl Into<String>) -> Self {
        self.inner.spdx = Some(spdx.into());
        self
    }

    /// URL to full license text;
    /// preferred over [Self::text] but below [Self::spdx].
    pub fn url(mut self, url: UriBuf) -> Self {
        self.inner.url = Some(url);
        self
    }

    /// Full license text;
    /// preferred over [Self::file] but below [Self::url].
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.inner.text = Some(text.into());
        self
    }

    /// Relative path to a file containing the license text;
    /// preferred over [Self::path] but below [Self::file].
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.inner.file = Some(file.into());
        self
    }

    /// Relative path to a zarr node with license metadata;
    /// least preferred option.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.inner.path = Some(path.into());
        self
    }

    /// Build the license item.
    /// Fails if no specifiers are set.
    pub fn build(mut self) -> Result<License, String> {
        if self.short {
            let mut none = false;
            if self.inner.spdx.is_some() {
                none = true;
            }
            if none {
                self.inner.url = None;
            } else if self.inner.url.is_some() {
                none = true;
            }
            if none {
                self.inner.text = None;
            } else if self.inner.text.is_some() {
                none = true;
            }
            if none {
                self.inner.file = None;
            } else if self.inner.file.is_some() {
                none = true;
            }
            if none {
                self.inner.path = None;
            }
        }
        self.inner.try_into()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zarrs_conventions::{
        AttributesBuilder, AttributesParser, ConventionId, DEFAULT_ZARR_CONVENTION_REGISTRY,
        ZarrConventionImpl,
    };

    use crate::License;

    #[test]
    fn is_registered() {
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY
                .contains(&ConventionId::Uuid(License::DEFINITION.uuid))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SchemaUrl(
                License::DEFINITION.schema_url.to_owned()
            ))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SpecUrl(
                License::DEFINITION.spec_url.to_owned()
            ))
        );
    }

    #[test]
    fn pass_expected() {
        let value = json!({
            "zarr_conventions": [{"uuid": License::DEFINITION.uuid}],
            "license": {"spdx": "MIT"}
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        let _license: License = parser.parse_nested().unwrap().unwrap();
    }

    #[test]
    fn fail_empty() {
        let value = json!({
            "zarr_conventions": [{"uuid": License::DEFINITION.uuid}],
            "license": {}
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        assert!(parser.parse_nested::<License>().is_err());
    }

    #[test]
    fn can_build() {
        let license = License::builder().spdx("MIT").build().unwrap();
        let mut builder = AttributesBuilder::default();
        builder.add_nested(&license).unwrap();
        let _attrs = builder.build().unwrap();
        println!("{_attrs:#}");
    }
}
