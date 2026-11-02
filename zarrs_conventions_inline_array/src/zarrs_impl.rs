//! Integration with the [`zarrs`] crate.

//!
//! An [`InlineArray`] can be converted into an [`InlineArrayBytes`],
//! which holds the decoded [`ArrayBytes`] alongside the zarrs [`DataType`] and shape.
//! Element data can then be read out with any [`FromArrayBytes`] type
//! (e.g. `Vec<T>`, `ndarray::ArrayD<T>`, [`ArrayBytes`]),
//! or created from any [`IntoArrayBytes`] type.
//!
//! ```
//! use zarrs_conventions_inline_array::{InlineArray, InlineArrayBytes, Metadata};
//!
//! let inline = InlineArray::builder([1u8, 2, 3, 4, 5, 6])
//!     .unwrap()
//!     .with_data_type(Metadata::new("uint8"))
//!     .with_shape(vec![2, 3])
//!     .unwrap();
//! let inline: InlineArray = inline.try_into().unwrap();
//!
//! let bytes = InlineArrayBytes::try_from(&inline).unwrap();
//! let values: Vec<u8> = bytes.to_data().unwrap();
//! assert_eq!(values, vec![1, 2, 3, 4, 5, 6]);
//!
//! let roundtripped = InlineArray::try_from(&bytes).unwrap();
//! assert_eq!(roundtripped.values(), inline.values());
//! ```
use zarrs::array::{
    ArrayBytes, ArrayBytesOffsets, DataType, DataTypeSize, FillValue, FillValueMetadata,
    FromArrayBytes, IntoArrayBytes,
};
use zarrs::metadata::{Configuration as ZarrsConfiguration, v3::MetadataV3};
use zarrs::plugin::{ExtensionName, ZarrVersion};

use crate::{FillValue as JsonValue, InlineArray, Metadata, Result};

/// Array data held as zarrs [`ArrayBytes`], with its [`DataType`] and shape.
#[derive(Debug, Clone)]
pub struct InlineArrayBytes {
    bytes: ArrayBytes<'static>,
    data_type: DataType,
    shape: Vec<u64>,
}

impl InlineArrayBytes {
    /// Wrap existing [`ArrayBytes`], validating them against the data type and shape.
    pub fn new(bytes: ArrayBytes<'static>, data_type: DataType, shape: Vec<u64>) -> Result<Self> {
        let n: u64 = shape.iter().product();
        bytes.validate(n, &data_type).map_err(|e| e.to_string())?;
        Ok(Self {
            bytes,
            data_type,
            shape,
        })
    }

    /// Create from any [`IntoArrayBytes`] type (e.g. `Vec<T>`, `&[T]`, `ndarray::Array`).
    pub fn from_data<'a>(
        data: impl IntoArrayBytes<'a>,
        data_type: DataType,
        shape: Vec<u64>,
    ) -> Result<Self> {
        let bytes = data
            .into_array_bytes(&data_type)
            .map_err(|e| e.to_string())?
            .into_owned();
        Self::new(bytes, data_type, shape)
    }

    /// The underlying array bytes.
    pub fn bytes(&self) -> &ArrayBytes<'static> {
        &self.bytes
    }

    /// Consume and return the underlying array bytes.
    pub fn into_bytes(self) -> ArrayBytes<'static> {
        self.bytes
    }

    /// The zarrs data type.
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// The array shape.
    pub fn shape(&self) -> &[u64] {
        &self.shape
    }

    /// Convert into any [`FromArrayBytes`] type, cloning the bytes.
    pub fn to_data<T: FromArrayBytes>(&self) -> Result<T> {
        T::from_array_bytes(self.bytes.clone(), &self.shape, &self.data_type)
            .map_err(|e| e.to_string())
    }

    /// Convert into any [`FromArrayBytes`] type.
    pub fn into_data<T: FromArrayBytes>(self) -> Result<T> {
        T::from_array_bytes(self.bytes, &self.shape, &self.data_type).map_err(|e| e.to_string())
    }
}

impl Metadata {
    /// Resolve into a zarrs [`DataType`].
    pub fn to_zarrs_data_type(&self) -> Result<DataType> {
        let meta = if self.configuration.is_empty() {
            MetadataV3::new(self.name.clone())
        } else {
            MetadataV3::new_with_configuration(
                self.name.clone(),
                ZarrsConfiguration::from(self.configuration.clone()),
            )
        };
        DataType::from_metadata(&meta).map_err(|e| e.to_string())
    }

    /// Create from a zarrs [`DataType`].
    pub fn from_zarrs_data_type(data_type: &DataType) -> Result<Self> {
        let name = data_type
            .name(ZarrVersion::V3)
            .ok_or_else(|| format!("data type {data_type} has no Zarr V3 name"))?;
        let configuration: serde_json::Map<_, _> = data_type.configuration(ZarrVersion::V3).into();
        Ok(Metadata {
            name: name.into_owned(),
            configuration,
        })
    }
}

impl InlineArray {
    /// Decode the inline values into [`InlineArrayBytes`].
    pub fn to_array_bytes(&self) -> Result<InlineArrayBytes> {
        self.try_into()
    }

    /// Read the inline values as any [`FromArrayBytes`] type.
    pub fn to_data<T: FromArrayBytes>(&self) -> Result<T> {
        self.to_array_bytes()?.into_data()
    }
}

impl TryFrom<&InlineArray> for InlineArrayBytes {
    type Error = crate::Error;

    fn try_from(array: &InlineArray) -> Result<Self> {
        let data_type = array.data_type.to_zarrs_data_type()?;
        let elements = array
            .values
            .iter()
            .map(|v| json_to_element(&data_type, v))
            .collect::<Result<Vec<_>>>()?;
        let bytes = encode_elements(&data_type, elements)?;
        let shape = array.shape.iter().map(|s| *s as u64).collect();
        Self::new(bytes, data_type, shape)
    }
}

impl TryFrom<InlineArray> for InlineArrayBytes {
    type Error = crate::Error;

    fn try_from(array: InlineArray) -> Result<Self> {
        (&array).try_into()
    }
}

/// Creates an [`InlineArray`] with no dimension names or attributes.
impl TryFrom<&InlineArrayBytes> for InlineArray {
    type Error = crate::Error;

    fn try_from(value: &InlineArrayBytes) -> Result<Self> {
        let values = decode_elements(&value.data_type, &value.bytes)?
            .iter()
            .map(|el| element_to_json(&value.data_type, el))
            .collect::<Result<Vec<_>>>()?;
        let shape = value
            .shape
            .iter()
            .map(|s| usize::try_from(*s).map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>>>()?;
        crate::InlineArrayBuilder::new(values)
            .with_data_type(Metadata::from_zarrs_data_type(&value.data_type)?)
            .with_shape(shape)?
            .try_into()
    }
}

/// Creates an [`InlineArray`] with no dimension names or attributes.
impl TryFrom<InlineArrayBytes> for InlineArray {
    type Error = crate::Error;

    fn try_from(value: InlineArrayBytes) -> Result<Self> {
        (&value).try_into()
    }
}

/// Convert a JSON value into the byte representation of one element.
fn json_to_element(data_type: &DataType, value: &JsonValue) -> Result<FillValue> {
    let meta: FillValueMetadata =
        serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
    data_type
        .fill_value(&meta, ZarrVersion::V3)
        .map_err(|e| format!("invalid value {value} for data type {data_type}: {e}"))
}

/// Convert the byte representation of one element into a JSON value.
fn element_to_json(data_type: &DataType, element: &FillValue) -> Result<JsonValue> {
    let meta = data_type
        .metadata_fill_value(element)
        .map_err(|e| format!("cannot represent element for data type {data_type}: {e}"))?;
    serde_json::to_value(meta).map_err(|e| e.to_string())
}

/// Combine per-element bytes into [`ArrayBytes`].
fn encode_elements(data_type: &DataType, elements: Vec<FillValue>) -> Result<ArrayBytes<'static>> {
    if let Some(opt) = data_type.as_optional() {
        let inner_type = opt.data_type();
        let mut mask = Vec::with_capacity(elements.len());
        let mut inner = Vec::with_capacity(elements.len());
        for el in elements {
            if opt.is_fill_value_null(&el) {
                mask.push(0u8);
                inner.push(placeholder_element(inner_type));
            } else {
                mask.push(1u8);
                inner.push(FillValue::new(opt.fill_value_inner_bytes(&el).to_vec()));
            }
        }
        return Ok(encode_elements(inner_type, inner)?.with_optional_mask(mask));
    }

    match data_type.size() {
        DataTypeSize::Fixed(size) => {
            let mut bytes = Vec::with_capacity(size * elements.len());
            for el in &elements {
                if el.size() != size {
                    return Err(format!(
                        "element has {} bytes, expected {size} for data type {data_type}",
                        el.size()
                    ));
                }
                bytes.extend_from_slice(el.as_ne_bytes());
            }
            Ok(ArrayBytes::new_flen(bytes))
        }
        DataTypeSize::Variable => {
            let mut bytes = Vec::new();
            let mut offsets = Vec::with_capacity(elements.len() + 1);
            offsets.push(0);
            for el in &elements {
                bytes.extend_from_slice(el.as_ne_bytes());
                offsets.push(bytes.len());
            }
            let offsets = ArrayBytesOffsets::new(offsets).map_err(|e| e.to_string())?;
            ArrayBytes::new_vlen(bytes, offsets).map_err(|e| e.to_string())
        }
    }
}

/// Element bytes used in the data of a null optional element.
fn placeholder_element(data_type: &DataType) -> FillValue {
    if data_type.as_optional().is_some() {
        FillValue::new_optional_null()
    } else {
        match data_type.size() {
            DataTypeSize::Fixed(size) => FillValue::new(vec![0; size]),
            DataTypeSize::Variable => FillValue::new(vec![]),
        }
    }
}

/// Split [`ArrayBytes`] into per-element bytes.
fn decode_elements(data_type: &DataType, bytes: &ArrayBytes<'_>) -> Result<Vec<FillValue>> {
    match bytes {
        ArrayBytes::Optional(opt_bytes) => {
            let opt = data_type
                .as_optional()
                .ok_or_else(|| format!("optional bytes for non-optional data type {data_type}"))?;
            let inner = decode_elements(opt.data_type(), opt_bytes.data())?;
            Ok(inner
                .into_iter()
                .zip(opt_bytes.mask().iter())
                .map(|(el, valid)| {
                    if *valid == 0 {
                        FillValue::new_optional_null()
                    } else {
                        el.into_optional()
                    }
                })
                .collect())
        }
        ArrayBytes::Fixed(raw) => {
            let DataTypeSize::Fixed(size) = data_type.size() else {
                return Err(format!(
                    "fixed-length bytes for variable-length data type {data_type}"
                ));
            };
            if size == 0 {
                return Err("zero-sized data types are not supported".to_string());
            }
            Ok(raw
                .chunks_exact(size)
                .map(|c| FillValue::new(c.to_vec()))
                .collect())
        }
        ArrayBytes::Variable(vlen) => Ok(vlen
            .offsets()
            .windows(2)
            .map(|w| FillValue::new(vlen.bytes()[w[0]..w[1]].to_vec()))
            .collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InlineArrayBuilder;

    fn build(builder: InlineArrayBuilder) -> InlineArray {
        builder.try_into().unwrap()
    }

    #[test]
    fn default_float64() {
        let arr = build(InlineArray::builder([1.0f64, 2.5, -3.0]).unwrap());
        let values: Vec<f64> = arr.to_data().unwrap();
        assert_eq!(values, vec![1.0, 2.5, -3.0]);
    }

    #[test]
    fn integer_roundtrip() {
        let arr = build(
            InlineArray::builder([1i32, -2, 3, 4, 5, 6])
                .unwrap()
                .with_data_type(Metadata::new("int32"))
                .with_shape(vec![3, 2])
                .unwrap(),
        );
        let bytes = arr.to_array_bytes().unwrap();
        assert_eq!(bytes.shape(), &[3, 2]);
        let values: Vec<i32> = bytes.to_data().unwrap();
        assert_eq!(values, vec![1, -2, 3, 4, 5, 6]);

        let back = InlineArray::try_from(&bytes).unwrap();
        assert_eq!(back.values(), arr.values());
        assert_eq!(back.shape(), arr.shape());
        assert_eq!(back.data_type().name, "int32");
    }

    #[test]
    fn from_into_array_bytes() {
        let bytes = InlineArrayBytes::from_data(
            vec![true, false, true],
            zarrs::array::data_type::bool(),
            vec![3],
        )
        .unwrap();
        let arr = InlineArray::try_from(bytes).unwrap();
        assert_eq!(
            arr.values(),
            &[
                serde_json::json!(true),
                serde_json::json!(false),
                serde_json::json!(true)
            ]
        );
    }

    #[test]
    fn string_roundtrip() {
        let arr = build(
            InlineArray::builder(["a", "bcd", ""])
                .unwrap()
                .with_data_type(Metadata::new("string")),
        );
        let values: Vec<String> = arr.to_data().unwrap();
        assert_eq!(values, vec!["a", "bcd", ""]);
        let back = InlineArray::try_from(arr.to_array_bytes().unwrap()).unwrap();
        assert_eq!(back.values(), arr.values());
    }

    #[test]
    fn invalid_value() {
        let arr = build(
            InlineArray::builder(["not a number"])
                .unwrap()
                .with_data_type(Metadata::new("uint8")),
        );
        assert!(arr.to_array_bytes().is_err());
    }

    #[test]
    fn unknown_data_type() {
        let arr = build(
            InlineArray::builder([1])
                .unwrap()
                .with_data_type(Metadata::new("not_a_real_type")),
        );
        assert!(arr.to_array_bytes().is_err());
    }
}
