#![doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
use zarrs_conventions::{
    ConventionDefinition, NestedRepr, ZarrConventionImpl,
    iref::{Uri, UriBuf, uri},
    register_zarr_conventions, uuid,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct License(Vec<LicenseItem>);

impl FromIterator<LicenseItem> for License {
    fn from_iter<T: IntoIterator<Item = LicenseItem>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn builder() -> LicenseItemBuilder {
        LicenseItemBuilder::default()
    }

    pub fn spdx(&self) -> Option<&str> {
        self.0.spdx.as_deref()
    }

    pub fn url(&self) -> Option<&Uri> {
        self.0.url.as_deref()
    }

    pub fn text(&self) -> Option<&str> {
        self.0.text.as_deref()
    }

    pub fn file(&self) -> Option<&str> {
        self.0.file.as_deref()
    }

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

    /// SPDX license identifier; RECOMMENDED.
    pub fn spdx(mut self, spdx: impl Into<String>) -> Self {
        self.inner.spdx = Some(spdx.into());
        self
    }

    /// URL to full license text.
    pub fn url(mut self, url: UriBuf) -> Self {
        self.inner.url = Some(url);
        self
    }

    /// Full license text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.inner.text = Some(text.into());
        self
    }

    /// Relative path to a file containing the license text.
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.inner.file = Some(file.into());
        self
    }

    /// Relative path to a zarr node with license metadata.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.inner.path = Some(path.into());
        self
    }

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
