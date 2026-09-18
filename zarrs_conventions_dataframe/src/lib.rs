#![doc = include_str!("../README.md")]
use serde::{Deserialize, Serialize};
pub use zarrs_conventions;
use zarrs_conventions::{
    ConventionDefinition, PrefixedRepr, ZarrConventionImpl, iref::uri, register_zarr_conventions,
    uuid,
};

/// Single license applicable to the data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "DataFrameSerde", into = "DataFrameSerde")]
pub struct DataFrameMeta {
    index: Option<usize>,
    columns: Vec<String>,
}

impl DataFrameMeta {
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            index: None,
            columns,
        }
    }

    pub fn new_with_index(columns: Vec<String>, index: usize) -> Result<Self, String> {
        if index >= columns.len() {
            return Err(format!(
                "Index {} is out of bounds for columns of length {}",
                index,
                columns.len()
            ));
        }
        Ok(Self {
            index: Some(index),
            columns,
        })
    }

    pub fn index(&self) -> Option<&str> {
        Some(
            self.columns
                .get(self.index?)
                .expect("dataframe index not found in columns")
                .as_str(),
        )
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }
}

impl From<DataFrameMeta> for DataFrameSerde {
    fn from(value: DataFrameMeta) -> Self {
        let index = value.index.and_then(|i| value.columns.get(i).cloned());
        Self {
            index,
            columns: value.columns,
        }
    }
}

impl TryFrom<DataFrameSerde> for DataFrameMeta {
    type Error = String;

    fn try_from(value: DataFrameSerde) -> Result<Self, Self::Error> {
        let index = match value.index {
            Some(ref idx) => Some(
                value
                    .columns
                    .iter()
                    .position(|c| c == idx)
                    .ok_or_else(|| format!("Index column '{}' not found in columns", idx))?,
            ),
            None => None,
        };
        Ok(DataFrameMeta {
            index,
            columns: value.columns,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataFrameSerde {
    index: Option<String>,
    columns: Vec<String>,
}

impl ZarrConventionImpl for DataFrameMeta {
    const DEFINITION: ConventionDefinition = ConventionDefinition {
        uuid: uuid::uuid!("1f1b36d2-3185-6da8-91f6-9a0823af2d33"),
        schema_url: uri!(
            "https://raw.githubusercontent.com/DOES_NOT_EXIST/zarr-dataframe/refs/tags/v1/schema.json"
        ),
        spec_url: uri!("https://github.com/DOES_NOT_EXIST/zarr-dataframe/blob/v1/README.md"),
        name: "df",
        description: "A collection of 1d-interpreted arrays to be used as a dataframe",
    };
}

impl PrefixedRepr for DataFrameMeta {
    const PREFIX: &'static str = "df:";
}

register_zarr_conventions!(DataFrameMeta);

#[cfg(test)]
mod tests {
    use serde_json::json;
    use zarrs_conventions::{
        AttributesBuilder, AttributesParser, ConventionId, DEFAULT_ZARR_CONVENTION_REGISTRY,
        ZarrConventionImpl,
    };

    use crate::DataFrameMeta;

    #[test]
    fn is_registered() {
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY
                .contains(&ConventionId::Uuid(DataFrameMeta::DEFINITION.uuid))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SchemaUrl(
                DataFrameMeta::DEFINITION.schema_url.to_owned()
            ))
        );
        assert!(
            DEFAULT_ZARR_CONVENTION_REGISTRY.contains(&ConventionId::SpecUrl(
                DataFrameMeta::DEFINITION.spec_url.to_owned()
            ))
        );
    }

    #[test]
    fn pass_expected() {
        let value = json!({
            "zarr_conventions": [{"uuid": DataFrameMeta::DEFINITION.uuid}],
            "df:index": "index",
            "df:columns": ["index", "a", "b", "c"]
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        let _license: DataFrameMeta = parser.parse_prefixed().unwrap().unwrap();
    }

    #[test]
    fn fail_empty() {
        let value = json!({
            "zarr_conventions": [{"uuid": DataFrameMeta::DEFINITION.uuid}],
        });
        let parser: AttributesParser = serde_json::from_value(value).unwrap();
        assert!(parser.parse_prefixed::<DataFrameMeta>().is_err());
    }

    #[test]
    fn can_build() {
        let meta = DataFrameMeta::new(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        let mut builder = AttributesBuilder::default();
        builder.add_prefixed(&meta).unwrap();
        let attributes = builder.build().unwrap();
        let parser: AttributesParser = serde_json::from_value(attributes).unwrap();
        let _license: DataFrameMeta = parser.parse_prefixed().unwrap().unwrap();
    }

    #[test]
    fn fail_index_oob() {
        let result = DataFrameMeta::new_with_index(vec!["a".to_string(), "b".to_string()], 2);
        assert!(result.is_err());
    }

    #[test]
    fn fail_unknown_index_deser() {
        let jso = json!({
            "index": "c",
            "columns": ["a", "b"]
        });
        let result: Result<DataFrameMeta, _> = serde_json::from_value(jso);
        assert!(result.is_err());
    }
}
