use std::hash::Hash;

use indexmap::{Equivalent, IndexSet};

/// Convert integer-encoded enums into their corresponding values.
#[derive(Debug, Clone)]
pub struct EnumDecoder<T> {
    values: Vec<T>,
}

impl<T> EnumDecoder<T> {
    /// Create a new decoder from a vector of values.
    ///
    /// Does not check for duplicates in the values.
    /// A decoder with duplicate values cannot be round-tripped through an encoder.
    pub fn new_unchecked(values: Vec<T>) -> Self {
        Self { values }
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn decode(&self, index: usize) -> Option<&T> {
        self.values.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }

    /// Return the largest index which can be decoded, or None if the decoder is empty.
    pub fn max_index(&self) -> Option<usize> {
        self.values.len().checked_sub(1)
    }
}

impl<T> IntoIterator for EnumDecoder<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<T: Hash + Eq> From<EnumEncoder<T>> for EnumDecoder<T> {
    fn from(encoder: EnumEncoder<T>) -> Self {
        Self {
            values: encoder.values.into_iter().collect(),
        }
    }
}

impl<T: Hash + Eq> From<EnumDecoder<T>> for EnumEncoder<T> {
    fn from(decoder: EnumDecoder<T>) -> Self {
        EnumEncoder {
            values: decoder.values.into_iter().collect(),
        }
    }
}

/// Convert enum values into integer.
///
/// Can also be used as a builder for an [EnumDecoder]:
///
/// ```
/// use zarrs_conventions_enum::{EnumEncoder, EnumDecoder};
///
/// let mut enc: EnumEncoder<&'static str> = Default::default();
/// enc.insert("b");
/// enc.insert("a");
/// let sorted = enc.sorted();
/// let dec = EnumDecoder::from(sorted);
/// ```
#[derive(Debug, Clone, Default)]
pub struct EnumEncoder<T> {
    values: IndexSet<T>,
}

impl<T: Hash + Eq> EnumEncoder<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: IndexSet::with_capacity(capacity),
        }
    }

    /// Returns the index of the value, inserting it if not present.
    ///
    /// The first item of the tuple is the index of the value,
    /// and the second item is true if the value was newly inserted.
    ///
    /// The resulting index should be ignored if the encoder is to be sorted later.
    pub fn insert(&mut self, value: T) -> (usize, bool) {
        self.values.insert_full(value)
    }

    /// Returns Some if the value is present, None if not.
    pub fn encode<Q>(&self, value: &Q) -> Option<usize>
    where
        Q: ?Sized + Hash + Equivalent<T>,
    {
        self.values.get_index_of(value)
    }

    pub fn sorted_by<F>(self, compare: F) -> Self
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        let mut values = self.values;
        values.sort_by(compare);
        Self { values }
    }

    pub fn sorted_by_key<K, F>(self, f: F) -> Self
    where
        K: Ord,
        F: FnMut(&T) -> K,
    {
        let mut values = self.values;
        values.sort_by_key(f);
        Self { values }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.iter()
    }
}

impl<T> IntoIterator for EnumEncoder<T> {
    type Item = T;
    type IntoIter = <indexmap::IndexSet<T> as std::iter::IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<T: Hash + Eq + Ord> EnumEncoder<T> {
    pub fn sorted(self) -> Self {
        let mut values = self.values;
        values.sort();
        Self { values }
    }
}

impl<V: Hash + Eq> FromIterator<V> for EnumEncoder<V> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        let mut builder = Self {
            values: IndexSet::default(),
        };
        for value in iter {
            builder.insert(value);
        }
        builder
    }
}
