use std::{
    borrow::{Borrow, Cow},
    fmt::Debug,
};

use crate::{RawArray, RawBsonRef, RawDocumentBuf};

use super::{document_buf::BindRawBsonRef, RawArrayIter};

/// An owned BSON array value (akin to [`std::path::PathBuf`]), backed by a buffer of raw BSON
/// bytes. This type can be used to construct owned array values, which can be used to append to
/// [`RawDocumentBuf`] or as a field in a [`Deserialize`](serde::Deserialize) struct.
///
/// Iterating over a [`RawArrayBuf`] yields either an error or a [`RawBson`](crate::raw::RawBson)
/// value that borrows from the original document without making any additional allocations.
/// ```
/// # use bson::error::Error;
/// use bson::raw::RawArrayBuf;
///
/// let mut array = RawArrayBuf::new();
/// array.push("a string");
/// array.push(12_i32);
///
/// let mut iter = array.into_iter();
///
/// let value = iter.next().unwrap()?;
/// assert_eq!(value.as_str(), Some("a string"));
///
/// let value = iter.next().unwrap()?;
/// assert_eq!(value.as_i32(), Some(12));
///
/// assert!(iter.next().is_none());
/// # Ok::<(), Error>(())
/// ```
///
/// This type implements [`Deref`](std::ops::Deref) to [`RawArray`], meaning that all methods on
/// [`RawArray`] are available on [`RawArrayBuf`] values as well. This includes [`RawArray::get`] or
/// any of the type-specific getters, such as [`RawArray::get_object_id`] or [`RawArray::get_str`].
/// Note that accessing elements is an O(N) operation, as it requires iterating through the document
/// from the beginning to find the requested key.
#[derive(Clone, PartialEq)]
pub struct RawArrayBuf {
    inner: RawDocumentBuf,
    len: usize,
}

impl RawArrayBuf {
    /// Construct a new, empty [`RawArrayBuf`].
    pub fn new() -> RawArrayBuf {
        Self {
            inner: RawDocumentBuf::new(),
            len: 0,
        }
    }

    /// Construct a new [`RawArrayBuf`] from the provided [`Vec`] of bytes.
    ///
    /// This involves a traversal of the array to count the values.
    pub(crate) fn from_raw_document_buf(doc: RawDocumentBuf) -> Self {
        let len = doc.iter().count();
        Self { inner: doc, len }
    }

    /// Append a value to the end of the array.
    ///
    /// ```
    /// # use bson::error::Error;
    /// use bson::raw::{cstr, RawArrayBuf, RawDocumentBuf};
    ///
    /// let mut array = RawArrayBuf::new();
    /// array.push("a string");
    /// array.push(12_i32);
    ///
    /// let mut doc = RawDocumentBuf::new();
    /// doc.append(cstr!("a key"), "a value");
    /// array.push(doc.clone());
    ///
    /// let mut iter = array.into_iter();
    ///
    /// let value = iter.next().unwrap()?;
    /// assert_eq!(value.as_str(), Some("a string"));
    ///
    /// let value = iter.next().unwrap()?;
    /// assert_eq!(value.as_i32(), Some(12));
    ///
    /// let value = iter.next().unwrap()?;
    /// assert_eq!(value.as_document(), Some(doc.as_ref()));
    ///
    /// assert!(iter.next().is_none());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn push(&mut self, value: impl BindRawBsonRef) {
        self.inner.append(
            super::CString::from_string_unchecked(self.len.to_string()),
            value,
        );
        self.len += 1;
    }
}

impl<B: BindRawBsonRef> FromIterator<B> for RawArrayBuf {
    fn from_iter<T: IntoIterator<Item = B>>(iter: T) -> Self {
        let mut array_buf = RawArrayBuf::new();
        for item in iter {
            array_buf.push(item);
        }
        array_buf
    }
}

impl Debug for RawArrayBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawArrayBuf")
            .field("data", &hex::encode(self.as_bytes()))
            .field("len", &self.len)
            .finish()
    }
}

impl std::ops::Deref for RawArrayBuf {
    type Target = RawArray;

    fn deref(&self) -> &Self::Target {
        RawArray::from_doc(&self.inner)
    }
}

impl AsRef<RawArray> for RawArrayBuf {
    fn as_ref(&self) -> &RawArray {
        RawArray::from_doc(&self.inner)
    }
}

impl Borrow<RawArray> for RawArrayBuf {
    fn borrow(&self) -> &RawArray {
        self.as_ref()
    }
}

impl<'a> IntoIterator for &'a RawArrayBuf {
    type IntoIter = RawArrayIter<'a>;
    type Item = super::Result<RawBsonRef<'a>>;

    fn into_iter(self) -> RawArrayIter<'a> {
        self.as_ref().into_iter()
    }
}

impl From<RawArrayBuf> for Cow<'_, RawArray> {
    fn from(rd: RawArrayBuf) -> Self {
        Cow::Owned(rd)
    }
}

impl<'a> From<&'a RawArrayBuf> for Cow<'a, RawArray> {
    fn from(rd: &'a RawArrayBuf) -> Self {
        Cow::Borrowed(rd.as_ref())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for RawArrayBuf {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(super::serde::OwnedOrBorrowedRawArray::deserialize(deserializer)?.into_owned())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for RawArrayBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl Default for RawArrayBuf {
    fn default() -> Self {
        Self::new()
    }
}

impl TryFrom<&crate::Array> for RawArrayBuf {
    type Error = crate::error::Error;

    fn try_from(value: &crate::Array) -> Result<Self, Self::Error> {
        Self::try_from(value.clone())
    }
}

impl TryFrom<crate::Array> for RawArrayBuf {
    type Error = crate::error::Error;

    fn try_from(value: crate::Array) -> Result<Self, Self::Error> {
        let mut tmp = RawArrayBuf::new();
        for val in value {
            let raw: super::RawBson = val.try_into()?;
            tmp.push(raw);
        }
        Ok(tmp)
    }
}
