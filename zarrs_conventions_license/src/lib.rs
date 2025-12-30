#![doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, NestedRepr, ZarrConventionImpl,
    iref::{Uri, UriBuf, uri},
    register_zarr_conventions, uuid,
};

/// Type representing zero or more licenses applicable to the data.
///
/// ```
/// use zarrs_conventions_license::{License, LicenseItem};
///
/// let single: License = LicenseItem::new_spdx("MIT").into();
/// let multi = License::from_iter([
///     LicenseItem::new_url("https://opensource.org/license/BSD-3-Clause".parse().unwrap()),
///     LicenseItem::new_spdx("Apache-2.0"),
/// ]);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct License(Vec<LicenseItem>);

impl AsRef<[LicenseItem]> for License {
    fn as_ref(&self) -> &[LicenseItem] {
        &self.0
    }
}

impl License {
    /// Get a mutable reference to the inner vec of license items.
    pub fn inner_mut(&mut self) -> &mut Vec<LicenseItem> {
        &mut self.0
    }
}

impl From<Vec<LicenseItem>> for License {
    fn from(value: Vec<LicenseItem>) -> Self {
        Self(value)
    }
}

impl IntoIterator for License {
    type Item = LicenseItem;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<LicenseItem> for License {
    fn from_iter<T: IntoIterator<Item = LicenseItem>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<LicenseItem> for License {
    fn from(value: LicenseItem) -> Self {
        Self(vec![value])
    }
}

/// Single license applicable to the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "LicenseItemInner", into = "LicenseItemInner")]
pub struct LicenseItem(LicenseItemInner);

impl From<LicenseItem> for LicenseItemInner {
    fn from(value: LicenseItem) -> Self {
        value.0
    }
}

impl TryFrom<LicenseItemInner> for LicenseItem {
    type Error = String;

    fn try_from(value: LicenseItemInner) -> Result<Self, Self::Error> {
        if value.spdx.is_none()
            && value.url.is_none()
            && value.text.is_none()
            && value.file.is_none()
            && value.path.is_none()
        {
            return Err("At least one field must be set for LicenseItem".to_string());
        }
        Ok(LicenseItem(value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LicenseItemInner {
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

impl LicenseItem {
    /// Builder for constructing a [LicenseItem].
    pub fn builder() -> LicenseItemBuilder {
        LicenseItemBuilder::default()
    }

    /// Create a new license item from an SPDX identifier.
    pub fn new_spdx<S: Into<String>>(identifier: S) -> Self {
        Self(LicenseItemInner { spdx: Some(identifier.into()), ..Default::default() })
    }

    /// License as an SPDX identifier.
    /// Should not be a multi-license expression.
    pub fn spdx(&self) -> Option<&str> {
        self.0.spdx.as_deref()
    }

    /// Create a new license item from a URL to the license text.
    pub fn new_url(url: UriBuf) -> Self {
        Self(LicenseItemInner { url: Some(url), ..Default::default() })
    }

    /// URL to the full license text.
    pub fn url(&self) -> Option<&Uri> {
        self.0.url.as_deref()
    }

    /// Create a new license item from a URL to the license text.
    pub fn new_text<S: Into<String>>(text: S) -> Self {
        Self(LicenseItemInner { text: Some(text.into()), ..Default::default() })
    }

    /// Full license text.
    pub fn text(&self) -> Option<&str> {
        self.0.text.as_deref()
    }

    /// Create a new license item from a relative path to an object containing the license text.
    pub fn new_file<S: Into<String>>(file: S) -> Self {
        Self(LicenseItemInner { file: Some(file.into()), ..Default::default() })
    }

    /// Relative path to an object containing the full license text.
    pub fn file(&self) -> Option<&str> {
        self.0.file.as_deref()
    }

    /// Create a new license item from a relative path to a zarr node with license metadata.
    pub fn new_path<S: Into<String>>(path: S) -> Self {
        Self(LicenseItemInner { path: Some(path.into()), ..Default::default() })
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

/// Builder for [LicenseItem]s, created by [LicenseItem::builder].
///
/// At least one license identifier must be set.
/// It is recommended to only set one.
/// In order of preference, `spdx > url > text > file > path`.
///
/// ## Examples
///
/// ```
/// use zarrs_conventions_license::LicenseItem;
///
/// let item = LicenseItem::builder().spdx("MIT").url("https://opensource.org/license/mit".parse().unwrap()).build().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct LicenseItemBuilder {
    inner: LicenseItemInner,
    short: bool,
}

impl Default for LicenseItemBuilder {
    fn default() -> Self {
        Self {
            inner: LicenseItemInner {
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

impl LicenseItemBuilder {
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
    pub fn build(mut self) -> Result<LicenseItem, String> {
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

    use crate::{License, LicenseItem};

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
            "license": [
                {"spdx": "MIT"},
            ]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        let _license: License = parser.parse_nested().unwrap().unwrap();
    }

    #[test]
    fn fail_empty() {
        let value = json!({
            "zarr_conventions": [{"uuid": License::DEFINITION.uuid}],
            "license": [
                {},
            ]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        assert!(parser.parse_nested::<License>().is_err());
    }

    #[test]
    fn can_build() {
        let license = License::from_iter([LicenseItem::builder().spdx("MIT").build().unwrap()]);
        let mut builder = AttributesBuilder::default();
        builder.add_nested(&license).unwrap();
        let _attrs = builder.build().unwrap();
        println!("{_attrs:#}");
    }
}
