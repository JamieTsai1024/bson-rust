//! Collection of helper functions for serializing to and deserializing from BSON using Serde
use serde::{de::Visitor, ser, Deserialize, Serialize, Serializer};
use std::{
    convert::TryFrom,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    result::Result,
};

#[doc(inline)]
pub use timestamp_as_u32::{
    deserialize as deserialize_timestamp_from_u32,
    serialize as serialize_timestamp_as_u32,
};
#[doc(inline)]
pub use u32_as_f64::{deserialize as deserialize_u32_from_f64, serialize as serialize_u32_as_f64};
#[doc(inline)]
pub use u32_as_timestamp::{
    deserialize as deserialize_u32_from_timestamp,
    serialize as serialize_u32_as_timestamp,
};
#[doc(inline)]
pub use u64_as_f64::{deserialize as deserialize_u64_from_f64, serialize as serialize_u64_as_f64};

#[cfg(feature = "uuid-1")]
#[doc(inline)]
pub use uuid_1_as_binary::{
    deserialize as deserialize_uuid_1_from_binary,
    serialize as serialize_uuid_1_as_binary,
};
#[cfg(feature = "uuid-1")]
#[doc(inline)]
pub use uuid_1_as_c_sharp_legacy_binary::{
    deserialize as deserialize_uuid_1_from_c_sharp_legacy_binary,
    serialize as serialize_uuid_1_as_c_sharp_legacy_binary,
};
#[cfg(feature = "uuid-1")]
#[doc(inline)]
pub use uuid_1_as_java_legacy_binary::{
    deserialize as deserialize_uuid_1_from_java_legacy_binary,
    serialize as serialize_uuid_1_as_java_legacy_binary,
};
#[cfg(feature = "uuid-1")]
#[doc(inline)]
pub use uuid_1_as_python_legacy_binary::{
    deserialize as deserialize_uuid_1_from_python_legacy_binary,
    serialize as serialize_uuid_1_as_python_legacy_binary,
};

/// Attempts to serialize a u32 as an i32. Errors if an exact conversion is not possible.
pub fn serialize_u32_as_i32<S: Serializer>(val: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    match i32::try_from(*val) {
        Ok(val) => serializer.serialize_i32(val),
        Err(_) => Err(ser::Error::custom(format!("cannot convert {} to i32", val))),
    }
}

/// Serializes a u32 as an i64.
pub fn serialize_u32_as_i64<S: Serializer>(val: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_i64(*val as i64)
}

/// Attempts to serialize a u64 as an i32. Errors if an exact conversion is not possible.
pub fn serialize_u64_as_i32<S: Serializer>(val: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    match i32::try_from(*val) {
        Ok(val) => serializer.serialize_i32(val),
        Err(_) => Err(ser::Error::custom(format!("cannot convert {} to i32", val))),
    }
}

/// Attempts to serialize a u64 as an i64. Errors if an exact conversion is not possible.
pub fn serialize_u64_as_i64<S: Serializer>(val: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    match i64::try_from(*val) {
        Ok(val) => serializer.serialize_i64(val),
        Err(_) => Err(ser::Error::custom(format!("cannot convert {} to i64", val))),
    }
}

#[cfg(feature = "serde_with-3")]
pub mod object_id {
    use crate::{macros::serde_conv_doc, oid::ObjectId};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{DeserializeAs, SerializeAs};

    serde_conv_doc!(
        /// Contains functions to serialize an ObjectId as a hex string and deserialize an
        /// ObjectId from a hex string
        /// ```rust
        /// # #[cfg(feature = "serde_with-3")]
        /// {
        /// # use serde::{Serialize, Deserialize};
        /// # use bson::serde_helpers::object_id;
        /// # use serde_with::serde_as;
        /// # use bson::oid::ObjectId;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Item {
        ///     #[serde_as(as = "object_id::AsHexString")]
        ///     pub id: ObjectId,
        /// }
        /// # }
        /// ```
        pub AsHexString,
        ObjectId,
        |oid: &ObjectId| -> Result<String, String> {
            Ok(oid.to_hex())
        },
        |hex: String| -> Result<ObjectId, String> {
            ObjectId::parse_str(&hex).map_err(|e| format!("Invalid ObjectId string, {}: {}", hex, e))
        }
    );

    serde_conv_doc!(
        /// Contains functions to serialize a hex string as an ObjectId and deserialize a
        /// hex string from an ObjectId
        /// ```rust
        /// # #[cfg(feature = "serde_with-3")]
        /// {
        /// # use serde::{Serialize, Deserialize};
        /// # use bson::serde_helpers::object_id;
        /// # use serde_with::serde_as;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Item {
        ///     #[serde_as(as = "object_id::FromHexString")]
        ///     pub id: String,
        /// }
        /// # }
        /// ```
        pub FromHexString,
        String,
        |hex: &String| -> Result<ObjectId, String> {
            ObjectId::parse_str(hex).map_err(|e| format!("Invalid ObjectId string, {}: {}", hex, e))
        },
        |oid: ObjectId| -> Result<String, String> {
            Ok(oid.to_hex())
        }
    );
}

/// Contains functions to serialize a u32 as an f64 (BSON double) and deserialize a
/// u32 from an f64 (BSON double).
///
/// ```rust
/// # use serde::{Serialize, Deserialize};
/// # use bson::serde_helpers::u32_as_f64;
/// #[derive(Serialize, Deserialize)]
/// struct FileInfo {
///     #[serde(with = "u32_as_f64")]
///     pub size_bytes: u32,
/// }
/// ```
pub mod u32_as_f64 {
    use serde::{de, Deserialize, Deserializer, Serializer};

    /// Deserializes a u32 from an f64 (BSON double). Errors if an exact conversion is not possible.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let f = f64::deserialize(deserializer)?;
        if (f - f as u32 as f64).abs() <= f64::EPSILON {
            Ok(f as u32)
        } else {
            Err(de::Error::custom(format!(
                "cannot convert f64 (BSON double) {} to u32",
                f
            )))
        }
    }

    /// Serializes a u32 as an f64 (BSON double).
    pub fn serialize<S: Serializer>(val: &u32, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(*val as f64)
    }
}

/// Contains functions to serialize a u64 as an f64 (BSON double) and deserialize a
/// u64 from an f64 (BSON double).
///
/// ```rust
/// # use serde::{Serialize, Deserialize};
/// # use bson::serde_helpers::u64_as_f64;
/// #[derive(Serialize, Deserialize)]
/// struct FileInfo {
///     #[serde(with = "u64_as_f64")]
///     pub size_bytes: u64,
/// }
/// ```
pub mod u64_as_f64 {
    use serde::{de, ser, Deserialize, Deserializer, Serializer};

    /// Deserializes a u64 from an f64 (BSON double). Errors if an exact conversion is not possible.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let f = f64::deserialize(deserializer)?;
        if (f - f as u64 as f64).abs() <= f64::EPSILON {
            Ok(f as u64)
        } else {
            Err(de::Error::custom(format!(
                "cannot convert f64 (BSON double) {} to u64",
                f
            )))
        }
    }

    /// Serializes a u64 as an f64 (BSON double). Errors if an exact conversion is not possible.
    pub fn serialize<S: Serializer>(val: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        if val < &u64::MAX && *val == *val as f64 as u64 {
            serializer.serialize_f64(*val as f64)
        } else {
            Err(ser::Error::custom(format!(
                "cannot convert u64 {} to f64 (BSON double)",
                val
            )))
        }
    }
}

/// Type converters for serializing and deserializing [`crate::DateTime`] using
/// [`serde_with::serde_as`].
///
/// ## Available converters
/// - [`datetime::AsRfc3339String`] — converts a [`crate::DateTime`] to and from an RFC 3339 string.
/// - [`datetime::FromRfc3339String`] — converts a RFC 3339 string to and from a
///   [`crate::DateTime`].
/// - [`datetime::FromI64`] — converts an `i64` to and from a [`crate::DateTime`].
/// - [`datetime::FromChrono04DateTime`] — converts a [`chrono::DateTime`] to and from a
///   [`crate::DateTime`].
/// - [`datetime::FromTime03OffsetDateTime`] — converts a [`time::OffsetDateTime`] to and from a
///   [`crate::DateTime`].
#[cfg(feature = "serde_with-3")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde_with-3")))]
pub mod datetime {
    use crate::{macros::serde_conv_doc, DateTime};
    use chrono::Utc;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde_with::{DeserializeAs, SerializeAs};
    use std::result::Result;

    serde_conv_doc!(
        /// Converts a [`DateTime`] to and from an RFC 3339 (ISO 8601) formatted string.
        /// ```rust
        /// # #[cfg(feature = "serde_with-3")]
        /// # {
        /// use bson::{serde_helpers::datetime, DateTime};
        /// use serde::{Serialize, Deserialize};
        /// use serde_with::serde_as;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Event {
        ///     #[serde_as(as = "datetime::AsRfc3339String")]
        ///     pub date: DateTime,
        /// }
        /// # }
        /// ```
        pub AsRfc3339String,
        DateTime,
        |date: &DateTime| -> Result<String, String> {
            date.try_to_rfc3339_string().map_err(|e| {
                format!("Cannot format DateTime {} as RFC 3339 string: {}", date, e)
            })
        },
        |string: String| -> Result<DateTime, String> {
            DateTime::parse_rfc3339_str(&string).map_err(|e| format!("Cannot format RFC 3339 string {} as DateTime: {}", string, e))
        }
    );

    serde_conv_doc!(
        /// Converts an RFC 3339 (ISO 8601) formatted string to and from a [`DateTime`].
        /// ```rust
        /// # #[cfg(feature = "serde_with-3")]
        /// # {
        /// use bson::serde_helpers::datetime;
        /// use serde::{Serialize, Deserialize};
        /// use serde_with::serde_as;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Event {
        ///     #[serde_as(as = "datetime::FromRfc3339String")]
        ///     pub date: String,
        /// }
        /// # }
        pub FromRfc3339String,
        String,
        |string: &String| -> Result<DateTime, String> {
            DateTime::parse_rfc3339_str(string).map_err(|e| format!("Cannot format RFC 3339 string {} as DateTime: {}", string, e))
        },
        |date: DateTime| -> Result<String, String> {
            date.try_to_rfc3339_string().map_err(|e| {
                format!("Cannot format DateTime {} as RFC 3339 string: {}", date, e)
            })
        }
    );

    serde_conv_doc!(
        /// Converts an `i64` integer to and from a [`DateTime`].
        ///
        /// The `i64` should represent milliseconds. See [`DateTime::from_millis`] for more details.
        /// ```rust
        /// # #[cfg(feature = "serde_with-3")]
        /// # {
        /// use bson::serde_helpers::datetime;
        /// use serde::{Serialize, Deserialize};
        /// use serde_with::serde_as;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Item {
        ///     #[serde_as(as = "datetime::FromI64")]
        ///     pub now: i64,
        /// }
        /// # }
        /// ```
        pub FromI64,
        i64,
        |value: &i64| -> Result<DateTime, String> {
            Ok(DateTime::from_millis(*value))
        },
        |date: DateTime| -> Result<i64, String> {
            Ok(date.timestamp_millis())
        }
    );

    #[cfg(feature = "chrono-0_4")]
    serde_conv_doc!(
        #[cfg_attr(docsrs, doc(cfg(feature = "chrono-0_4")))]
        /// Converts a [`chrono::DateTime`] to and from a [`DateTime`].
        /// ```rust
        /// # #[cfg(all(feature = "chrono-0_4", feature = "serde_with-3"))]
        /// # {
        /// use bson::serde_helpers::datetime;
        /// use serde::{Serialize, Deserialize};
        /// use serde_with::serde_as;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Event {
        ///     #[serde_as(as = "datetime::FromChrono04DateTime")]
        ///     pub date: chrono::DateTime<chrono::Utc>,
        /// }
        /// # }
        /// ```
        pub FromChrono04DateTime,
        chrono::DateTime<Utc>,
        |chrono_date: &chrono::DateTime<Utc>| -> Result<DateTime, String> {
            Ok(DateTime::from_chrono(*chrono_date))
        },
        |bson_date: DateTime| -> Result<chrono::DateTime<Utc>, String> {
            Ok(bson_date.to_chrono())
        }
    );

    #[cfg(feature = "time-0_3")]
    serde_conv_doc!(
        #[cfg_attr(docsrs, doc(cfg(feature = "time-0_3")))]
        /// Converts a [`time::OffsetDateTime`] to and from a [`DateTime`].
        /// ```rust
        /// # #[cfg(all(feature = "time-0_3", feature = "serde_with-3"))]
        /// # {
        /// use bson::serde_helpers::datetime;
        /// use serde::{Serialize, Deserialize};
        /// use serde_with::serde_as;
        /// #[serde_as]
        /// #[derive(Serialize, Deserialize)]
        /// struct Event {
        ///     #[serde_as(as = "datetime::FromTime03OffsetDateTime")]
        ///     pub date: time::OffsetDateTime,
        /// }
        /// # }
        /// ```
        pub FromTime03OffsetDateTime,
        time::OffsetDateTime,
        |value: &time::OffsetDateTime| -> Result<DateTime, String> {
            Ok(DateTime::from_time_0_3(*value))
        },
        |date: DateTime| -> Result<time::OffsetDateTime, String> {
            Ok(date.to_time_0_3())
        }
    );
}

#[allow(unused_macros)]
macro_rules! as_binary_mod {
    ($feat:meta, $uu:path) => {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};
        use std::result::Result;
        use $uu;

        /// Serializes a Uuid as a Binary.
        #[cfg_attr(docsrs, doc($feat))]
        pub fn serialize<S: Serializer>(val: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
            crate::uuid::Uuid::from(*val).serialize(serializer)
        }

        /// Deserializes a Uuid from a Binary.
        #[cfg_attr(docsrs, doc($feat))]
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
        where
            D: Deserializer<'de>,
        {
            let bson_uuid = crate::uuid::Uuid::deserialize(deserializer)?;
            Ok(bson_uuid.into())
        }
    };
}

/// Contains functions to serialize a [`uuid::Uuid`] as a [`crate::Binary`] and deserialize a
/// [`uuid::Uuid`] from a [`crate::Binary`].
///
/// ```rust
/// # #[cfg(feature = "uuid-1")]
/// # {
/// use serde::{Serialize, Deserialize};
/// use uuid::Uuid;
/// use bson::serde_helpers::uuid_1_as_binary;
///
/// #[derive(Serialize, Deserialize)]
/// struct Item {
///     #[serde(with = "uuid_1_as_binary")]
///     pub id: Uuid,
/// }
/// # }
/// ```
#[cfg(feature = "uuid-1")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid-1")))]
pub mod uuid_1_as_binary {
    as_binary_mod!(cfg(feature = "uuid-1"), uuid::Uuid);
}

#[allow(unused_macros)]
macro_rules! as_legacy_binary_mod {
    ($feat:meta, $uu:path, $rep:path) => {
        use crate::{uuid::UuidRepresentation, Binary};
        use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
        use std::result::Result;
        use $uu;

        /// Serializes a Uuid as a Binary in the legacy UUID format.
        #[cfg_attr(docsrs, doc($feat))]
        pub fn serialize<S: Serializer>(val: &Uuid, serializer: S) -> Result<S::Ok, S::Error> {
            let binary = Binary::from_uuid_with_representation(crate::uuid::Uuid::from(*val), $rep);
            binary.serialize(serializer)
        }

        /// Deserializes a Uuid from a Binary in the legacy UUID format.
        #[cfg_attr(docsrs, doc($feat))]
        pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
        where
            D: Deserializer<'de>,
        {
            let binary = Binary::deserialize(deserializer)?;
            let uuid = binary
                .to_uuid_with_representation($rep)
                .map_err(de::Error::custom)?;
            Ok(uuid.into())
        }
    };
}

/// Contains functions to serialize a [`uuid::Uuid`] to a [`crate::Binary`] in the legacy
/// Java driver UUID format and deserialize [`uuid::Uuid`] from a [`crate::Binary`] in the legacy
/// Java driver format.
///
/// ```rust
/// #[cfg(feature = "uuid-1")]
/// # {
/// use serde::{Serialize, Deserialize};
/// use uuid::Uuid;
/// use bson::serde_helpers::uuid_1_as_java_legacy_binary;
///
/// #[derive(Serialize, Deserialize)]
/// struct Item {
///     #[serde(with = "uuid_1_as_java_legacy_binary")]
///     pub id: Uuid,
/// }
/// # }
/// ```
#[cfg(feature = "uuid-1")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid-1")))]
pub mod uuid_1_as_java_legacy_binary {
    as_legacy_binary_mod!(
        cfg(feature = "uuid-1"),
        uuid::Uuid,
        UuidRepresentation::JavaLegacy
    );
}

/// Contains functions to serialize a [`uuid::Uuid`] to a [`crate::Binary`] in the legacy Python
/// driver UUID format and deserialize [`uuid::Uuid`] from a [`crate::Binary`] in the legacy Python
/// driver format.
///
/// ```rust
/// # #[cfg(feature = "uuid-1")]
/// # {
/// use serde::{Serialize, Deserialize};
/// use uuid::Uuid;
/// use bson::serde_helpers::uuid_1_as_python_legacy_binary;
///
/// #[derive(Serialize, Deserialize)]
/// struct Item {
///     #[serde(with = "uuid_1_as_python_legacy_binary")]
///     pub id: Uuid,
/// }
/// # }
/// ```
#[cfg(feature = "uuid-1")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid-1")))]
pub mod uuid_1_as_python_legacy_binary {
    as_legacy_binary_mod!(
        cfg(feature = "uuid-1"),
        uuid::Uuid,
        UuidRepresentation::PythonLegacy
    );
}

/// Contains functions to serialize a [`uuid::Uuid`] to a [`crate::Binary`] in the legacy C# driver
/// UUID format and deserialize [`uuid::Uuid`] from a [`crate::Binary`] in the legacy C# driver
/// format.
///
/// ```rust
/// # #[cfg(feature = "uuid-1")]
/// # {
/// use serde::{Serialize, Deserialize};
/// use uuid::Uuid;
/// use bson::serde_helpers::uuid_1_as_c_sharp_legacy_binary;
///
/// #[derive(Serialize, Deserialize)]
/// struct Item {
///     #[serde(with = "uuid_1_as_c_sharp_legacy_binary")]
///     pub id: Uuid,
/// }
/// # }
/// ```
#[cfg(feature = "uuid-1")]
#[cfg_attr(docsrs, doc(cfg(feature = "uuid-1")))]
pub mod uuid_1_as_c_sharp_legacy_binary {
    as_legacy_binary_mod!(
        cfg(feature = "uuid-1"),
        uuid::Uuid,
        UuidRepresentation::CSharpLegacy
    );
}

/// Contains functions to serialize a u32 as a bson::Timestamp and deserialize a u32 from a
/// bson::Timestamp. The u32 should represent seconds since the Unix epoch.
///
/// ```rust
/// # use serde::{Serialize, Deserialize};
/// # use bson::serde_helpers::u32_as_timestamp;
/// #[derive(Serialize, Deserialize)]
/// struct Event {
///     #[serde(with = "u32_as_timestamp")]
///     pub time: u32,
/// }
/// ```
pub mod u32_as_timestamp {
    use crate::{Bson, Timestamp};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::result::Result;

    /// Serializes a u32 as a bson::Timestamp.
    pub fn serialize<S: Serializer>(val: &u32, serializer: S) -> Result<S::Ok, S::Error> {
        let timestamp = Bson::Timestamp(Timestamp {
            time: *val,
            increment: 0,
        });
        timestamp.serialize(serializer)
    }

    /// Deserializes a u32 from a bson::Timestamp.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = Timestamp::deserialize(deserializer)?;
        Ok(timestamp.time)
    }
}

/// Contains functions to serialize a bson::Timestamp as a u32 and deserialize a bson::Timestamp
/// from a u32. The u32 should represent seconds since the Unix epoch. Serialization will return an
/// error if the Timestamp has a non-zero increment.
///
/// ```rust
/// # use serde::{Serialize, Deserialize};
/// # use bson::{serde_helpers::timestamp_as_u32, Timestamp};
/// #[derive(Serialize, Deserialize)]
/// struct Item {
///     #[serde(with = "timestamp_as_u32")]
///     pub timestamp: Timestamp,
/// }
/// ```
pub mod timestamp_as_u32 {
    use crate::Timestamp;
    use serde::{ser, Deserialize, Deserializer, Serializer};
    use std::result::Result;

    /// Serializes a bson::Timestamp as a u32. Returns an error if the conversion is lossy (i.e. the
    /// Timestamp has a non-zero increment).
    pub fn serialize<S: Serializer>(val: &Timestamp, serializer: S) -> Result<S::Ok, S::Error> {
        if val.increment != 0 {
            return Err(ser::Error::custom(
                "Cannot convert Timestamp with a non-zero increment to u32",
            ));
        }
        serializer.serialize_u32(val.time)
    }

    /// Deserializes a bson::Timestamp from a u32.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time = u32::deserialize(deserializer)?;
        Ok(Timestamp { time, increment: 0 })
    }
}

/// Wrapping a type in `HumanReadable` signals to the BSON serde integration that it and all
/// recursively contained types should be serialized to and deserialized from their human-readable
/// formats.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
#[repr(transparent)]
pub struct HumanReadable<T>(pub T);

pub(crate) const HUMAN_READABLE_NEWTYPE: &str = "$__bson_private_human_readable";

impl<T: Serialize> Serialize for HumanReadable<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(HUMAN_READABLE_NEWTYPE, &self.0)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for HumanReadable<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V<T>(PhantomData<fn() -> T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = HumanReadable<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("HumanReadable wrapper")
            }
            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                T::deserialize(deserializer).map(HumanReadable)
            }
        }
        deserializer.deserialize_newtype_struct(HUMAN_READABLE_NEWTYPE, V(PhantomData))
    }
}

impl<T: std::fmt::Display> std::fmt::Display for HumanReadable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for HumanReadable<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for HumanReadable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for HumanReadable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, R> AsRef<R> for HumanReadable<T>
where
    R: ?Sized,
    <HumanReadable<T> as Deref>::Target: AsRef<R>,
{
    fn as_ref(&self) -> &R {
        self.deref().as_ref()
    }
}

impl<T, R: ?Sized> AsMut<R> for HumanReadable<T>
where
    <HumanReadable<T> as Deref>::Target: AsMut<R>,
{
    fn as_mut(&mut self) -> &mut R {
        self.deref_mut().as_mut()
    }
}

// One could imagine passthrough Borrow impls; however, it turns out that can't be made to work
// because of the existing base library impl of Borrow<T> for T will conflict despite that not
// actually being possible to construct (https://github.com/rust-lang/rust/issues/50237).  So,
// sadly, Borrow impls for HumanReadable are deliberately omitted :(

/// Wrapper type for deserializing BSON bytes with invalid UTF-8 sequences.
///
/// Any invalid UTF-8 strings contained in the wrapped type will be replaced with the Unicode
/// replacement character. This wrapper type only has an effect when deserializing from BSON bytes.
///
/// This wrapper type has no impact on serialization. Serializing a `Utf8LossyDeserialization<T>`
/// will call the `serialize` method for the wrapped `T`.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
#[repr(transparent)]
pub struct Utf8LossyDeserialization<T>(pub T);

pub(crate) const UTF8_LOSSY_NEWTYPE: &str = "$__bson_private_utf8_lossy";

impl<T: Serialize> Serialize for Utf8LossyDeserialization<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Utf8LossyDeserialization<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct V<T>(PhantomData<fn() -> T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = Utf8LossyDeserialization<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Utf8Lossy wrapper")
            }
            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                T::deserialize(deserializer).map(Utf8LossyDeserialization)
            }
        }
        deserializer.deserialize_newtype_struct(UTF8_LOSSY_NEWTYPE, V(PhantomData))
    }
}

impl<T: std::fmt::Display> std::fmt::Display for Utf8LossyDeserialization<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<T> for Utf8LossyDeserialization<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Utf8LossyDeserialization<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Utf8LossyDeserialization<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, R> AsRef<R> for Utf8LossyDeserialization<T>
where
    R: ?Sized,
    <Utf8LossyDeserialization<T> as Deref>::Target: AsRef<R>,
{
    fn as_ref(&self) -> &R {
        self.deref().as_ref()
    }
}

impl<T, R: ?Sized> AsMut<R> for Utf8LossyDeserialization<T>
where
    <Utf8LossyDeserialization<T> as Deref>::Target: AsMut<R>,
{
    fn as_mut(&mut self) -> &mut R {
        self.deref_mut().as_mut()
    }
}
