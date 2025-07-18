#![allow(clippy::disallowed_names)]

use crate::{
    bson,
    cstr,
    deserialize_from_bson,
    deserialize_from_document,
    doc,
    oid::ObjectId,
    serde_helpers::{self, datetime, object_id, timestamp_as_u32, u32_as_timestamp},
    serialize_to_bson,
    serialize_to_document,
    spec::BinarySubtype,
    tests::LOCK,
    Binary,
    Bson,
    DateTime,
    Deserializer,
    Document,
    Serializer,
    Timestamp,
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::serde_as;

use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
};

#[test]
fn test_ser_vec() {
    let _guard = LOCK.run_concurrently();
    let vec = vec![1, 2, 3];

    let serializer = Serializer::new();
    let result = vec.serialize(serializer).unwrap();

    let expected = bson!([1, 2, 3]);
    assert_eq!(expected, result);
}

#[test]
fn test_ser_map() {
    let _guard = LOCK.run_concurrently();
    let mut map = BTreeMap::new();
    map.insert("x", 0);
    map.insert("y", 1);

    let serializer = Serializer::new();
    let result = map.serialize(serializer).unwrap();

    let expected = bson!({ "x": 0, "y": 1 });
    assert_eq!(expected, result);
}

#[test]
fn test_de_vec() {
    let _guard = LOCK.run_concurrently();
    let bson = bson!([1, 2, 3]);

    let deserializer = Deserializer::new(bson);
    let vec = Vec::<i32>::deserialize(deserializer).unwrap();

    let expected = vec![1, 2, 3];
    assert_eq!(expected, vec);
}

#[test]
fn test_de_map() {
    let _guard = LOCK.run_concurrently();
    let bson = bson!({ "x": 0, "y": 1 });

    let deserializer = Deserializer::new(bson);
    let map = BTreeMap::<String, i32>::deserialize(deserializer).unwrap();

    let mut expected = BTreeMap::new();
    expected.insert("x".to_string(), 0);
    expected.insert("y".to_string(), 1);
    assert_eq!(expected, map);
}

#[test]
fn test_ser_timestamp() {
    let _guard = LOCK.run_concurrently();
    use bson::Timestamp;

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    struct Foo {
        ts: Timestamp,
    }

    let foo = Foo {
        ts: Timestamp {
            time: 12,
            increment: 10,
        },
    };

    let x = serialize_to_bson(&foo).unwrap();
    assert_eq!(
        x.as_document().unwrap(),
        &doc! { "ts": Bson::Timestamp(Timestamp { time: 0x0000_000C, increment: 0x0000_000A }) }
    );

    let xfoo: Foo = deserialize_from_bson(x).unwrap();
    assert_eq!(xfoo, foo);
}

#[test]
fn test_de_timestamp() {
    let _guard = LOCK.run_concurrently();
    use bson::Timestamp;

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    struct Foo {
        ts: Timestamp,
    }

    let foo: Foo = deserialize_from_bson(Bson::Document(doc! {
        "ts": Bson::Timestamp(Timestamp { time: 0x0000_000C, increment: 0x0000_000A }),
    }))
    .unwrap();

    assert_eq!(
        foo.ts,
        Timestamp {
            time: 12,
            increment: 10
        }
    );
}

#[test]
fn test_ser_regex() {
    let _guard = LOCK.run_concurrently();
    use bson::Regex;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        regex: Regex,
    }

    let regex = Regex {
        pattern: cstr!("12").into(),
        options: cstr!("01").into(),
    };

    let foo = Foo {
        regex: regex.clone(),
    };

    let x = serialize_to_bson(&foo).unwrap();
    assert_eq!(
        x.as_document().unwrap(),
        &doc! { "regex": Bson::RegularExpression(regex) }
    );

    let xfoo: Foo = deserialize_from_bson(x).unwrap();
    assert_eq!(xfoo, foo);
}

#[test]
fn test_de_regex() {
    let _guard = LOCK.run_concurrently();
    use bson::Regex;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Foo {
        regex: Regex,
    }

    let regex = Regex {
        pattern: cstr!("12").into(),
        options: cstr!("01").into(),
    };

    let foo: Foo = deserialize_from_bson(Bson::Document(doc! {
        "regex": Bson::RegularExpression(regex.clone()),
    }))
    .unwrap();

    assert_eq!(foo.regex, regex);
}

#[test]
fn test_ser_code_with_scope() {
    let _guard = LOCK.run_concurrently();
    use bson::JavaScriptCodeWithScope;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        code_with_scope: JavaScriptCodeWithScope,
    }

    let code_with_scope = JavaScriptCodeWithScope {
        code: "x".into(),
        scope: doc! { "x": 12 },
    };

    let foo = Foo {
        code_with_scope: code_with_scope.clone(),
    };

    let x = serialize_to_bson(&foo).unwrap();
    assert_eq!(
        x.as_document().unwrap(),
        &doc! { "code_with_scope": Bson::JavaScriptCodeWithScope(code_with_scope) }
    );

    let xfoo: Foo = deserialize_from_bson(x).unwrap();
    assert_eq!(xfoo, foo);
}

#[test]
fn test_de_code_with_scope() {
    let _guard = LOCK.run_concurrently();
    use bson::JavaScriptCodeWithScope;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Foo {
        code_with_scope: JavaScriptCodeWithScope,
    }

    let code_with_scope = JavaScriptCodeWithScope {
        code: "x".into(),
        scope: doc! { "x": 12 },
    };

    let foo: Foo = deserialize_from_bson(Bson::Document(doc! {
        "code_with_scope": Bson::JavaScriptCodeWithScope(code_with_scope.clone()),
    }))
    .unwrap();

    assert_eq!(foo.code_with_scope, code_with_scope);
}

#[test]
fn test_ser_datetime() {
    let _guard = LOCK.run_concurrently();
    use crate::DateTime;

    #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    struct Foo {
        date: DateTime,
    }

    let now = DateTime::now();

    let foo = Foo { date: now };

    let x = serialize_to_bson(&foo).unwrap();
    assert_eq!(
        x.as_document().unwrap(),
        &doc! { "date": (Bson::DateTime(now)) }
    );

    let xfoo: Foo = deserialize_from_bson(x).unwrap();
    assert_eq!(xfoo, foo);
}

#[test]
fn test_binary_generic_roundtrip() {
    let _guard = LOCK.run_concurrently();
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct Foo {
        data: Bson,
    }

    let x = Foo {
        data: Bson::Binary(Binary {
            subtype: BinarySubtype::Generic,
            bytes: b"12345abcde".to_vec(),
        }),
    };

    let b = serialize_to_bson(&x).unwrap();
    assert_eq!(
        b.as_document().unwrap(),
        &doc! {"data": Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: b"12345abcde".to_vec() })}
    );

    let f = deserialize_from_bson::<Foo>(b).unwrap();
    assert_eq!(x, f);
}

#[test]
fn test_binary_non_generic_roundtrip() {
    let _guard = LOCK.run_concurrently();
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct Foo {
        data: Bson,
    }

    let x = Foo {
        data: Bson::Binary(Binary {
            subtype: BinarySubtype::BinaryOld,
            bytes: b"12345abcde".to_vec(),
        }),
    };

    let b = serialize_to_bson(&x).unwrap();
    assert_eq!(
        b.as_document().unwrap(),
        &doc! {"data": Bson::Binary(Binary { subtype: BinarySubtype::BinaryOld, bytes: b"12345abcde".to_vec() })}
    );

    let f = deserialize_from_bson::<Foo>(b).unwrap();
    assert_eq!(x, f);
}

#[test]
fn test_binary_helper_generic_roundtrip() {
    let _guard = LOCK.run_concurrently();
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct Foo {
        data: Binary,
    }

    let x = Foo {
        data: Binary {
            subtype: BinarySubtype::Generic,
            bytes: b"12345abcde".to_vec(),
        },
    };

    let b = serialize_to_bson(&x).unwrap();
    assert_eq!(
        b.as_document().unwrap(),
        &doc! {"data": Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: b"12345abcde".to_vec() })}
    );

    let f = deserialize_from_bson::<Foo>(b).unwrap();
    assert_eq!(x, f);
}

#[test]
fn test_binary_helper_non_generic_roundtrip() {
    let _guard = LOCK.run_concurrently();
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct Foo {
        data: Binary,
    }

    let x = Foo {
        data: Binary {
            subtype: BinarySubtype::BinaryOld,
            bytes: b"12345abcde".to_vec(),
        },
    };

    let b = serialize_to_bson(&x).unwrap();
    assert_eq!(
        b.as_document().unwrap(),
        &doc! {"data": Bson::Binary(Binary { subtype: BinarySubtype::BinaryOld, bytes: b"12345abcde".to_vec() })}
    );

    let f = deserialize_from_bson::<Foo>(b).unwrap();
    assert_eq!(x, f);
}

#[test]
fn test_byte_vec() {
    let _guard = LOCK.run_concurrently();
    #[derive(Serialize, Debug, Eq, PartialEq)]
    pub struct AuthChallenge<'a> {
        #[serde(with = "serde_bytes")]
        pub challenge: &'a [u8],
    }

    let x = AuthChallenge {
        challenge: b"18762b98b7c34c25bf9dc3154e4a5ca3",
    };

    let b = serialize_to_bson(&x).unwrap();
    assert_eq!(
        b,
        Bson::Document(
            doc! { "challenge": (Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: x.challenge.to_vec() }))}
        )
    );
}

#[test]
fn test_serde_bytes() {
    let _guard = LOCK.run_concurrently();
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    pub struct Foo {
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    let x = Foo {
        data: b"12345abcde".to_vec(),
    };

    let b = serialize_to_bson(&x).unwrap();
    assert_eq!(
        b.as_document().unwrap(),
        &doc! {"data": Bson::Binary(Binary { subtype: BinarySubtype::Generic, bytes: b"12345abcde".to_vec() })}
    );

    let f = deserialize_from_bson::<Foo>(b).unwrap();
    assert_eq!(x, f);
}

#[test]
fn test_serde_newtype_struct() {
    let _guard = LOCK.run_concurrently();
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Email(String);

    let email_1 = Email(String::from("bson@serde.rs"));
    let b = serialize_to_bson(&email_1).unwrap();
    assert_eq!(b, Bson::String(email_1.0));

    let s = String::from("root@localho.st");
    let de = Bson::String(s.clone());
    let email_2 = deserialize_from_bson::<Email>(de).unwrap();
    assert_eq!(email_2, Email(s));
}

#[test]
fn test_serde_tuple_struct() {
    let _guard = LOCK.run_concurrently();
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Name(String, String); // first, last

    let name_1 = Name(String::from("Graydon"), String::from("Hoare"));
    let b = serialize_to_bson(&name_1).unwrap();
    assert_eq!(b, bson!([name_1.0.clone(), name_1.1]));

    let (first, last) = (String::from("Donald"), String::from("Knuth"));
    let de = bson!([first.clone(), last.clone()]);
    let name_2 = deserialize_from_bson::<Name>(de).unwrap();
    assert_eq!(name_2, Name(first, last));
}

#[test]
fn test_serde_newtype_variant() {
    let _guard = LOCK.run_concurrently();
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(tag = "type", content = "value")]
    enum Number {
        Int(i64),
        Float(f64),
    }

    let n = 42;
    let num_1 = Number::Int(n);
    let b = serialize_to_bson(&num_1).unwrap();
    assert_eq!(b, bson!({ "type": "Int", "value": n }));

    let x = 1337.0;
    let de = bson!({ "type": "Float", "value": x });
    let num_2 = deserialize_from_bson::<Number>(de).unwrap();
    assert_eq!(num_2, Number::Float(x));
}

#[test]
fn test_serde_tuple_variant() {
    let _guard = LOCK.run_concurrently();
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Point {
        TwoDim(f64, f64),
        ThreeDim(f64, f64, f64),
    }

    #[allow(clippy::approx_constant)]
    let (x1, y1) = (3.14, -2.71);
    let p1 = Point::TwoDim(x1, y1);
    let b = serialize_to_bson(&p1).unwrap();
    assert_eq!(b, bson!({ "TwoDim": [x1, y1] }));

    let (x2, y2, z2) = (0.0, -13.37, 4.2);
    let de = bson!({ "ThreeDim": [x2, y2, z2] });
    let p2 = deserialize_from_bson::<Point>(de).unwrap();
    assert_eq!(p2, Point::ThreeDim(x2, y2, z2));
}

#[test]
fn test_ser_db_pointer() {
    let _guard = LOCK.run_concurrently();
    use bson::DbPointer;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        db_pointer: DbPointer,
    }

    let db_pointer = Bson::try_from(json!({
        "$dbPointer": {
            "$ref": "db.coll",
            "$id": { "$oid": "507f1f77bcf86cd799439011" },
        }
    }))
    .unwrap();

    let db_pointer = db_pointer.as_db_pointer().unwrap();

    let foo = Foo {
        db_pointer: db_pointer.clone(),
    };

    let x = serialize_to_bson(&foo).unwrap();
    assert_eq!(
        x.as_document().unwrap(),
        &doc! {"db_pointer": Bson::DbPointer(db_pointer.clone()) }
    );

    let xfoo: Foo = deserialize_from_bson(x).unwrap();
    assert_eq!(xfoo, foo);
}

#[test]
fn test_de_db_pointer() {
    let _guard = LOCK.run_concurrently();
    use bson::DbPointer;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Foo {
        db_pointer: DbPointer,
    }

    let db_pointer = Bson::try_from(json!({
        "$dbPointer": {
            "$ref": "db.coll",
            "$id": { "$oid": "507f1f77bcf86cd799439011" },
        }
    }))
    .unwrap();
    let db_pointer = db_pointer.as_db_pointer().unwrap();

    let foo: Foo = deserialize_from_bson(Bson::Document(
        doc! {"db_pointer": Bson::DbPointer(db_pointer.clone())},
    ))
    .unwrap();

    assert_eq!(foo.db_pointer, db_pointer.clone());
}

#[cfg(feature = "uuid-1")]
#[test]
fn test_serde_legacy_uuid_1() {
    use uuid::Uuid;

    let _guard = LOCK.run_concurrently();

    #[derive(Serialize, Deserialize)]
    struct Foo {
        #[serde(with = "serde_helpers::uuid_1_as_java_legacy_binary")]
        java_legacy: Uuid,
        #[serde(with = "serde_helpers::uuid_1_as_python_legacy_binary")]
        python_legacy: Uuid,
        #[serde(with = "serde_helpers::uuid_1_as_c_sharp_legacy_binary")]
        csharp_legacy: Uuid,
    }
    let uuid = Uuid::parse_str("00112233445566778899AABBCCDDEEFF").unwrap();
    let foo = Foo {
        java_legacy: uuid,
        python_legacy: uuid,
        csharp_legacy: uuid,
    };

    let x = serialize_to_bson(&foo).unwrap();
    assert_eq!(
        x.as_document().unwrap(),
        &doc! {
            "java_legacy": Bson::Binary(Binary{
                subtype:BinarySubtype::UuidOld,
                bytes: hex::decode("7766554433221100FFEEDDCCBBAA9988").unwrap(),
            }),
            "python_legacy": Bson::Binary(Binary{
                subtype:BinarySubtype::UuidOld,
                bytes: hex::decode("00112233445566778899AABBCCDDEEFF").unwrap(),
            }),
            "csharp_legacy": Bson::Binary(Binary{
                subtype:BinarySubtype::UuidOld,
                bytes: hex::decode("33221100554477668899AABBCCDDEEFF").unwrap(),
            })
        }
    );

    let foo: Foo = deserialize_from_bson(x).unwrap();
    assert_eq!(foo.java_legacy, uuid);
    assert_eq!(foo.python_legacy, uuid);
    assert_eq!(foo.csharp_legacy, uuid);
}

#[test]
fn test_de_uuid_extjson_string() {
    let _guard = LOCK.run_concurrently();

    let uuid_bson_bytes =
        hex::decode("1D000000057800100000000473FFD26444B34C6990E8E7D1DFC035D400").unwrap();
    let uuid_document = Document::decode_from_reader(uuid_bson_bytes.as_slice()).unwrap();
    let expected_uuid_bson = Bson::from_extended_document(uuid_document);

    let ext_json_uuid = "{\"x\" : { \"$uuid\" : \"73ffd264-44b3-4c69-90e8-e7d1dfc035d4\"}}";
    let actual_uuid_bson: Bson = serde_json::from_str(ext_json_uuid).unwrap();

    assert_eq!(actual_uuid_bson, expected_uuid_bson);
}

#[test]
fn test_de_date_extjson_number() {
    let _guard = LOCK.run_concurrently();

    let ext_json_canonical = r#"{ "$date": { "$numberLong": "1136239445000" } }"#;
    let expected_date_bson: Bson = serde_json::from_str(ext_json_canonical).unwrap();

    let ext_json_legacy_java = r#"{ "$date": 1136239445000 }"#;
    let actual_date_bson: Bson = serde_json::from_str(ext_json_legacy_java).unwrap();

    assert_eq!(actual_date_bson, expected_date_bson);
}

#[test]
fn test_de_oid_string() {
    let _guard = LOCK.run_concurrently();

    #[derive(Debug, Deserialize)]
    struct Foo {
        pub oid: ObjectId,
    }

    let foo: Foo = serde_json::from_str("{ \"oid\": \"507f1f77bcf86cd799439011\" }").unwrap();
    let oid = ObjectId::parse_str("507f1f77bcf86cd799439011").unwrap();
    assert_eq!(foo.oid, oid);
}

#[test]
fn test_serialize_deserialize_unsigned_numbers() {
    let _guard = LOCK.run_concurrently();

    let num = 1;
    let json = format!("{{ \"num\": {} }}", num);
    let doc: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.get_i32("num").unwrap(), num);

    let num = i32::MAX as u64 + 1;
    let json = format!("{{ \"num\": {} }}", num);
    let doc: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc.get_i64("num").unwrap(), num as i64);

    let num = u64::MAX;
    let json = format!("{{ \"num\": {} }}", num);
    let doc_result: Result<Document, serde_json::Error> = serde_json::from_str(&json);
    assert!(doc_result.is_err());
}

#[test]
fn test_unsigned_helpers() {
    let _guard = LOCK.run_concurrently();

    #[derive(Serialize)]
    struct A {
        #[serde(serialize_with = "serde_helpers::serialize_u32_as_i32")]
        num_1: u32,
        #[serde(serialize_with = "serde_helpers::serialize_u64_as_i32")]
        num_2: u64,
    }

    let a = A { num_1: 1, num_2: 2 };
    let doc = serialize_to_document(&a).unwrap();
    assert!(doc.get_i32("num_1").unwrap() == 1);
    assert!(doc.get_i32("num_2").unwrap() == 2);

    let a = A {
        num_1: u32::MAX,
        num_2: 1,
    };
    let doc_result = serialize_to_document(&a);
    assert!(doc_result.is_err());

    let a = A {
        num_1: 1,
        num_2: u64::MAX,
    };
    let doc_result = serialize_to_document(&a);
    assert!(doc_result.is_err());

    #[derive(Serialize)]
    struct B {
        #[serde(serialize_with = "serde_helpers::serialize_u32_as_i64")]
        num_1: u32,
        #[serde(serialize_with = "serde_helpers::serialize_u64_as_i64")]
        num_2: u64,
    }

    let b = B {
        num_1: u32::MAX,
        num_2: i64::MAX as u64,
    };
    let doc = serialize_to_document(&b).unwrap();
    assert!(doc.get_i64("num_1").unwrap() == u32::MAX as i64);
    assert!(doc.get_i64("num_2").unwrap() == i64::MAX);

    let b = B {
        num_1: 1,
        num_2: i64::MAX as u64 + 1,
    };
    let doc_result = serialize_to_document(&b);
    assert!(doc_result.is_err());

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct F {
        #[serde(with = "serde_helpers::u32_as_f64")]
        num_1: u32,
        #[serde(with = "serde_helpers::u64_as_f64")]
        num_2: u64,
    }

    let f = F {
        num_1: 101,
        num_2: 12345,
    };
    let doc = serialize_to_document(&f).unwrap();
    assert!((doc.get_f64("num_1").unwrap() - 101.0).abs() < f64::EPSILON);
    assert!((doc.get_f64("num_2").unwrap() - 12345.0).abs() < f64::EPSILON);

    let back: F = deserialize_from_document(doc).unwrap();
    assert_eq!(back, f);

    let f = F {
        num_1: 1,
        // f64 cannot represent many large integers exactly, u64::MAX included
        num_2: u64::MAX,
    };
    let doc_result = serialize_to_document(&f);
    assert!(doc_result.is_err());

    let f = F {
        num_1: 1,
        num_2: u64::MAX - 255,
    };
    let doc_result = serialize_to_document(&f);
    assert!(doc_result.is_err());
}

#[test]
#[cfg(feature = "serde_with-3")]
fn test_datetime_rfc3339_string_helpers() {
    let _guard = LOCK.run_concurrently();

    #[serde_as]
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct A {
        #[serde_as(as = "datetime::AsRfc3339String")]
        pub date: DateTime,

        #[serde_as(as = "Option<datetime::AsRfc3339String>")]
        pub date_optional_none: Option<DateTime>,

        #[serde_as(as = "Option<datetime::AsRfc3339String>")]
        pub date_optional_some: Option<DateTime>,

        #[serde_as(as = "Vec<datetime::AsRfc3339String>")]
        pub date_vector: Vec<DateTime>,
    }

    let iso = "1996-12-20T00:39:57Z";
    let date = DateTime::parse_rfc3339_str(iso).unwrap();
    let a = A {
        date,
        date_optional_none: None,
        date_optional_some: Some(date),
        date_vector: vec![date],
    };

    // Serialize the struct to BSON
    let doc = serialize_to_document(&a).unwrap();

    // Validate serialized data
    assert_eq!(
        doc.get_str("date").unwrap(),
        iso,
        "Expected serialized date to match original date from RFC 3339 string."
    );

    assert_eq!(
        doc.get("date_optional_none"),
        Some(&Bson::Null),
        "Expected serialized date_optional_none to be None."
    );

    assert_eq!(
        doc.get("date_optional_some"),
        Some(&Bson::String(iso.to_string())),
        "Expected serialized date_optional_some to match original."
    );

    let date_vector = doc
        .get_array("date_vector")
        .expect("Expected serialized date_vector to be a BSON array.");
    let expected_date_vector: Vec<Bson> = vec![Bson::String(date.try_to_rfc3339_string().unwrap())];
    assert_eq!(
        date_vector, &expected_date_vector,
        "Expected each serialized element in date_vector to match the original."
    );

    // Validate deserialized data
    let a_deserialized: A = deserialize_from_document(doc).unwrap();
    assert_eq!(
        a_deserialized, a,
        "Deserialized struct does not match original."
    );

    // Validate deserializing error case with an invalid DateTime string
    let invalid_doc = doc! {
        "date": "not_a_valid_date",
        "date_optional_none": Bson::Null,
        "date_optional_some": "also_invalid_date",
        "date_vector": ["bad1", "bad2"]
    };
    let result: Result<A, _> = deserialize_from_document(invalid_doc);
    assert!(
        result.is_err(),
        "Deserialization should fail for invalid DateTime strings"
    );
    let err_string = format!("{:?}", result.unwrap_err());
    assert!(
        err_string.contains("BSON error"),
        "Expected error message to mention BSON error: {}",
        err_string
    );

    #[serde_as]
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct B {
        #[serde_as(as = "datetime::FromRfc3339String")]
        pub date: String,

        #[serde_as(as = "Option<datetime::FromRfc3339String>")]
        pub date_optional_none: Option<String>,

        #[serde_as(as = "Option<datetime::FromRfc3339String>")]
        pub date_optional_some: Option<String>,

        #[serde_as(as = "Vec<datetime::FromRfc3339String>")]
        pub date_vector: Vec<String>,
    }

    let date = DateTime::now();
    let b = B {
        date: date.try_to_rfc3339_string().unwrap(),
        date_optional_none: None,
        date_optional_some: Some(date.try_to_rfc3339_string().unwrap()),
        date_vector: vec![date.try_to_rfc3339_string().unwrap()],
    };

    // Serialize the struct to BSON
    let doc = serialize_to_document(&b).unwrap();

    // Validate serialized data
    assert_eq!(
        *doc.get_datetime("date").unwrap(),
        date,
        "Expected serialized date to be a BSON DateTime."
    );

    assert_eq!(
        doc.get("date_optional_none"),
        Some(&Bson::Null),
        "Expected serialized date_optional_none to be None."
    );

    assert_eq!(
        doc.get("date_optional_some"),
        Some(&Bson::DateTime(date)),
        "Expected serialized date_optional_some to match original."
    );

    let date_vector = doc
        .get_array("date_vector")
        .expect("Expected serialized date_vector to be a BSON array.");
    let expected_date_vector: Vec<Bson> = vec![Bson::DateTime(date)];
    assert_eq!(
        date_vector, &expected_date_vector,
        "Expected each serialized element in date_vector match the original."
    );

    // Validate deserialized data
    let b_deserialized: B = deserialize_from_document(doc).unwrap();
    assert_eq!(
        b_deserialized, b,
        "Deserialized struct does not match original."
    );

    // Validate serializing error case with an invalid DateTime string
    let invalid_date = "invalid_date";
    let bad_b = B {
        date: invalid_date.to_string(),
        date_optional_none: None,
        date_optional_some: Some(invalid_date.to_string()),
        date_vector: vec![invalid_date.to_string()],
    };
    let result = serialize_to_document(&bad_b);
    assert!(
        result.is_err(),
        "Serialization should fail for invalid DateTime strings"
    );
    let err_string = format!("{:?}", result.unwrap_err());
    assert!(
        err_string.contains("BSON error"),
        "Expected error message to mention BSON error: {}",
        err_string
    );
}

#[test]
#[cfg(feature = "serde_with-3")]
fn test_datetime_i64_helper() {
    let _guard = LOCK.run_concurrently();

    #[serde_as]
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct A {
        #[serde_as(as = "datetime::FromI64")]
        date: i64,

        #[serde_as(as = "Option<datetime::FromI64>")]
        date_optional_none: Option<i64>,

        #[serde_as(as = "Option<datetime::FromI64>")]
        date_optional_some: Option<i64>,

        #[serde_as(as = "Vec<datetime::FromI64>")]
        date_vector: Vec<i64>,
    }

    let date = DateTime::now();
    let a = A {
        date: date.timestamp_millis(),
        date_optional_none: None,
        date_optional_some: Some(date.timestamp_millis()),
        date_vector: vec![date.timestamp_millis()],
    };

    // Serialize the struct to BSON
    let doc = serialize_to_document(&a).unwrap();

    // Validate serialized data
    assert_eq!(
        doc.get_datetime("date").unwrap(),
        &date,
        "Expected serialized date to match original date."
    );

    assert_eq!(
        doc.get("date_optional_none"),
        Some(&Bson::Null),
        "Expected serialized date_optional_none to be None."
    );

    assert_eq!(
        doc.get("date_optional_some"),
        Some(&Bson::DateTime(date)),
        "Expected serialized date_optional_some to match original."
    );

    let date_vector = doc
        .get_array("date_vector")
        .expect("Expected serialized date_vector to be a BSON array.");
    let expected_date_vector: Vec<Bson> = vec![Bson::DateTime(date)];
    assert_eq!(
        date_vector, &expected_date_vector,
        "Expected each serialized element in date_vector match the original."
    );

    // Validate deserialized data
    let a_deserialized: A = deserialize_from_document(doc).unwrap();
    assert_eq!(
        a_deserialized, a,
        "Deserialized struct does not match original."
    );
}

#[test]
#[cfg(all(feature = "chrono-0_4", feature = "serde_with-3"))]
fn test_datetime_chrono04_datetime_helper() {
    let _guard = LOCK.run_concurrently();

    use std::str::FromStr;

    #[serde_as]
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct A {
        #[serde_as(as = "datetime::FromChrono04DateTime")]
        pub date: chrono::DateTime<chrono::Utc>,

        #[serde_as(as = "Option<datetime::FromChrono04DateTime>")]
        pub date_optional_none: Option<chrono::DateTime<chrono::Utc>>,

        #[serde_as(as = "Option<datetime::FromChrono04DateTime>")]
        pub date_optional_some: Option<chrono::DateTime<chrono::Utc>>,

        #[serde_as(as = "Vec<datetime::FromChrono04DateTime>")]
        pub date_vector: Vec<chrono::DateTime<chrono::Utc>>,
    }

    let iso = "1996-12-20T00:39:57Z";
    let date: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_str(iso).unwrap();
    let a: A = A {
        date,
        date_optional_none: None,
        date_optional_some: Some(date),
        date_vector: vec![date],
    };

    // Serialize the struct to BSON
    let doc = serialize_to_document(&a).unwrap();

    // Validate serialized data
    assert_eq!(
        doc.get_datetime("date").unwrap().to_chrono(),
        date,
        "Expected serialized date to match original date."
    );

    assert_eq!(
        doc.get("date_optional_none"),
        Some(&Bson::Null),
        "Expected serialized date_optional_none to be None."
    );

    assert_eq!(
        doc.get("date_optional_some"),
        Some(&Bson::DateTime(DateTime::from_chrono(date))),
        "Expected serialized date_optional_some to match original."
    );

    let date_vector = doc
        .get_array("date_vector")
        .expect("Expected serialized date_vector to be a BSON array.");
    let expected_date_vector: Vec<Bson> = vec![Bson::DateTime(date.into())];
    assert_eq!(
        date_vector, &expected_date_vector,
        "Expected each serialized element in date_vector to be a BSON DateTime matching the \
         original."
    );

    // Validate deserialized data
    let a_deserialized: A = deserialize_from_document(doc).unwrap();
    assert_eq!(
        a_deserialized, a,
        "Deserialized struct does not match original."
    );
}

#[test]
#[cfg(all(feature = "time-0_3", feature = "serde_with-3"))]
fn test_datetime_time03_offset_datetime_helper() {
    use time::OffsetDateTime;
    let _guard = LOCK.run_concurrently();

    #[serde_as]
    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct A {
        #[serde_as(as = "datetime::FromTime03OffsetDateTime")]
        pub date: OffsetDateTime,

        #[serde_as(as = "Option<datetime::FromTime03OffsetDateTime>")]
        pub date_optional_none: Option<OffsetDateTime>,

        #[serde_as(as = "Option<datetime::FromTime03OffsetDateTime>")]
        pub date_optional_some: Option<OffsetDateTime>,

        #[serde_as(as = "Vec<datetime::FromTime03OffsetDateTime>")]
        pub date_vector: Vec<OffsetDateTime>,
    }

    let date = DateTime::now();
    let a: A = A {
        date: date.to_time_0_3(),
        date_optional_none: None,
        date_optional_some: Some(date.to_time_0_3()),
        date_vector: vec![date.to_time_0_3()],
    };

    // Serialize the struct to BSON
    let doc = serialize_to_document(&a).unwrap();

    // Validate serialized data
    assert_eq!(
        doc.get_datetime("date").unwrap(),
        &date,
        "Expected serialized date to match original date."
    );

    assert_eq!(
        doc.get("date_optional_none"),
        Some(&Bson::Null),
        "Expected serialized date_optional_none to be None."
    );

    assert_eq!(
        doc.get("date_optional_some"),
        Some(&Bson::DateTime(date)),
        "Expected serialized date_optional_some to match original."
    );

    let date_vector = doc
        .get_array("date_vector")
        .expect("Expected serialized date_vector to be a BSON array.");
    let expected_date_vector: Vec<Bson> = vec![Bson::DateTime(date)];
    assert_eq!(
        date_vector, &expected_date_vector,
        "Expected each serialized element in date_vector to be a BSON DateTime matching the \
         original."
    );

    // Validate deserialized data
    let a_deserialized: A = deserialize_from_document(doc).unwrap();
    assert_eq!(
        a_deserialized, a,
        "Deserialized struct does not match original."
    );
}

#[test]
fn test_oid_helpers() {
    let _guard = LOCK.run_concurrently();

    #[cfg(feature = "serde_with-3")]
    {
        #[serde_as]
        #[derive(Serialize, Deserialize)]
        struct A {
            #[serde_as(as = "object_id::FromHexString")]
            oid: String,

            #[serde_as(as = "Option<object_id::FromHexString>")]
            oid_optional_none: Option<String>,

            #[serde_as(as = "Option<object_id::FromHexString>")]
            oid_optional_some: Option<String>,

            #[serde_as(as = "Vec<object_id::FromHexString>")]
            oid_vector: Vec<String>,
        }

        let oid = ObjectId::new();
        let a = A {
            oid: oid.to_string(),
            oid_optional_none: None,
            oid_optional_some: Some(oid.to_string()),
            oid_vector: vec![oid.to_string()],
        };

        // Serialize the struct to BSON
        let doc = serialize_to_document(&a).unwrap();

        // Validate serialized data
        assert_eq!(
            doc.get_object_id("oid").unwrap(),
            oid,
            "Expected serialized oid to match original ObjectId."
        );

        assert_eq!(
            doc.get("oid_optional_none"),
            Some(&Bson::Null),
            "Expected serialized oid_optional_none to be None."
        );

        match doc.get("oid_optional_some") {
            Some(Bson::ObjectId(value)) => {
                assert_eq!(
                    *value, oid,
                    "Expected serialized oid_optional_some to match original ObjectId."
                )
            }
            _ => {
                panic!("Expected serialized oid_optional_some to be a BSON ObjectId.")
            }
        }

        let oid_vector = doc
            .get_array("oid_vector")
            .expect("Expected serialized oid_vector to be a BSON array.");
        let expected_oid_vector: Vec<Bson> = a
            .oid_vector
            .into_iter()
            .map(|oid| Bson::ObjectId(ObjectId::parse_str(oid).unwrap()))
            .collect();
        assert_eq!(
            oid_vector, &expected_oid_vector,
            "Expected each serialized element in oid_vector match the original."
        );

        // Deserialize the BSON back to the struct
        let a_deserialized: A = deserialize_from_document(doc).unwrap();

        // Validate deserialized data
        assert_eq!(
            a_deserialized.oid,
            oid.to_string(),
            "Expected deserialized oid to match the original."
        );

        assert_eq!(
            a_deserialized.oid_optional_some,
            Some(oid.to_string()),
            "Expected deserialized oid_optional_some to match the original."
        );

        assert_eq!(
            a_deserialized.oid_vector,
            vec![oid.to_string()],
            "Expected deserialized oid_vector to match the original."
        );

        // Validate serializing error case with an invalid ObjectId string
        let invalid_oid = "invalid_oid";
        let bad_a = A {
            oid: invalid_oid.to_string(),
            oid_optional_none: None,
            oid_optional_some: Some(invalid_oid.to_string()),
            oid_vector: vec![invalid_oid.to_string()],
        };
        let result = serialize_to_document(&bad_a);
        assert!(
            result.is_err(),
            "Deserialization should fail for invalid ObjectId strings"
        );
        let err_string = format!("{:?}", result.unwrap_err());
        assert!(
            err_string.contains("BSON error"),
            "Expected error message to mention BSON error: {}",
            err_string
        );

        #[serde_as]
        #[derive(Serialize, Deserialize, Debug)]
        struct B {
            #[serde_as(as = "object_id::AsHexString")]
            oid: ObjectId,

            #[serde_as(as = "Option<object_id::AsHexString>")]
            oid_optional_none: Option<ObjectId>,

            #[serde_as(as = "Option<object_id::AsHexString>")]
            oid_optional_some: Option<ObjectId>,

            #[serde_as(as = "Vec<object_id::AsHexString>")]
            oid_vector: Vec<ObjectId>,
        }

        let oid = ObjectId::new();
        let b = B {
            oid,
            oid_optional_none: None,
            oid_optional_some: Some(oid),
            oid_vector: vec![oid],
        };

        // Serialize the struct to BSON
        let doc = serialize_to_document(&b).unwrap();

        // Validate serialized data
        assert_eq!(
            doc.get_str("oid").unwrap(),
            oid.to_hex(),
            "Expected serialized oid to match original ObjectId as hex string."
        );

        assert_eq!(
            doc.get("oid_optional_none"),
            Some(&Bson::Null),
            "Expected serialized oid_optional_none to be None."
        );

        match doc.get("oid_optional_some") {
            Some(Bson::String(value)) => {
                assert_eq!(
                    *value,
                    oid.to_hex(),
                    "Expected serialized oid_optional_some to match original ObjectId."
                )
            }
            _ => {
                panic!("Expected serialized oid_optional_some to be a BSON String.")
            }
        }

        let oid_vector = doc
            .get_array("oid_vector")
            .expect("Expected serialized oid_vector to be a BSON array.");
        let expected_oid_vector: Vec<Bson> = b
            .oid_vector
            .into_iter()
            .map(|oid| Bson::String(oid.to_hex()))
            .collect();
        assert_eq!(
            oid_vector, &expected_oid_vector,
            "Expected each serialized element in oid_vector match the original."
        );

        // Deserialize the BSON back to the struct
        let b_deserialized: B = deserialize_from_document(doc).unwrap();

        // Validate deserialized data
        assert_eq!(
            b_deserialized.oid, oid,
            "Expected deserialized oid to match the original."
        );

        assert_eq!(
            b_deserialized.oid_optional_none, None,
            "Expected deserialized oid_optional_none to be None."
        );

        assert_eq!(
            b_deserialized.oid_optional_some,
            Some(oid),
            "Expected deserialized oid_optional_some to match the original."
        );

        assert_eq!(
            b_deserialized.oid_vector,
            vec![oid],
            "Expected deserialized oid_vector to match the original.."
        );

        // Validate deserializing error case with an invalid ObjectId string
        let invalid_doc = doc! {
            "oid": "not_a_valid_oid",
            "oid_optional_none": Bson::Null,
            "oid_optional_some": "also_invalid_oid",
            "oid_vector": ["bad1", "bad2"]
        };
        let result: Result<B, _> = deserialize_from_document(invalid_doc);
        assert!(
            result.is_err(),
            "Deserialization should fail for invalid ObjectId strings"
        );
        let err_string = format!("{:?}", result.unwrap_err());
        assert!(
            err_string.contains("BSON error"),
            "Expected error message to mention BSON error: {}",
            err_string
        );
    }
}

#[test]
#[cfg(feature = "uuid-1")]
fn test_uuid_1_helpers() {
    use serde_helpers::uuid_1_as_binary;
    use uuid::Uuid;

    let _guard = LOCK.run_concurrently();

    #[derive(Serialize, Deserialize)]
    struct A {
        #[serde(with = "uuid_1_as_binary")]
        uuid: Uuid,
    }

    let uuid = Uuid::parse_str("936DA01F9ABD4d9d80C702AF85C822A8").unwrap();
    let a = A { uuid };
    let doc = serialize_to_document(&a).unwrap();
    match doc.get("uuid").unwrap() {
        Bson::Binary(bin) => {
            assert_eq!(bin.subtype, BinarySubtype::Uuid);
            assert_eq!(bin.bytes, uuid.as_bytes());
        }
        _ => panic!("expected Bson::Binary"),
    }
    let a: A = deserialize_from_document(doc).unwrap();
    assert_eq!(a.uuid, uuid);
}

#[test]
fn test_timestamp_helpers() {
    let _guard = LOCK.run_concurrently();

    #[derive(Deserialize, Serialize)]
    struct A {
        #[serde(with = "u32_as_timestamp")]
        pub time: u32,
    }

    let time = 12345;
    let a = A { time };
    let doc = serialize_to_document(&a).unwrap();
    let timestamp = doc.get_timestamp("time").unwrap();
    assert_eq!(timestamp.time, time);
    assert_eq!(timestamp.increment, 0);
    let a: A = deserialize_from_document(doc).unwrap();
    assert_eq!(a.time, time);

    #[derive(Deserialize, Serialize)]
    struct B {
        #[serde(with = "timestamp_as_u32")]
        pub timestamp: Timestamp,
    }

    let time = 12345;
    let timestamp = Timestamp { time, increment: 0 };
    let b = B { timestamp };
    let val = serde_json::to_value(b).unwrap();
    assert_eq!(val["timestamp"], time);
    let b: B = serde_json::from_value(val).unwrap();
    assert_eq!(b.timestamp, timestamp);

    let timestamp = Timestamp {
        time: 12334,
        increment: 1,
    };
    let b = B { timestamp };
    assert!(serde_json::to_value(b).is_err());
}

#[test]
fn large_dates() {
    let _guard = LOCK.run_concurrently();

    let json = json!({ "d": { "$date": { "$numberLong": i64::MAX.to_string() } } });
    let d = serde_json::from_value::<Document>(json.clone()).unwrap();
    assert_eq!(d.get_datetime("d").unwrap(), &DateTime::MAX);
    let d: Bson = json.try_into().unwrap();
    assert_eq!(
        d.as_document().unwrap().get_datetime("d").unwrap(),
        &DateTime::MAX
    );

    let json = json!({ "d": { "$date": { "$numberLong": i64::MIN.to_string() } } });
    let d = serde_json::from_value::<Document>(json.clone()).unwrap();
    assert_eq!(d.get_datetime("d").unwrap(), &DateTime::MIN);
    let d: Bson = json.try_into().unwrap();
    assert_eq!(
        d.as_document().unwrap().get_datetime("d").unwrap(),
        &DateTime::MIN
    );
}

#[test]
fn fuzz_regression_00() {
    let buf: &[u8] = &[227, 0, 35, 4, 2, 0, 255, 255, 255, 127, 255, 255, 255, 47];
    let _ = crate::deserialize_from_slice::<Document>(buf);
}

#[cfg(feature = "serde_path_to_error")]
mod serde_path_to_error {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct Foo {
        one: Bar,
        two: Bar,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Bar {
        value: u64,
    }

    #[test]
    fn de() {
        let src = doc! {
            "one": {
                "value": 42,
            },
            "two": {
                "value": "hello",
            },
        };
        let result: Result<Foo, _> = crate::deserialize_from_document(src);
        assert!(result.is_err());
        let path = result.unwrap_err().path.unwrap();
        assert_eq!(path.to_string(), "two.value");
    }

    #[test]
    fn de_raw() {
        let src = rawdoc! {
            "one": {
                "value": 42,
            },
            "two": {
                "value": "hello",
            },
        }
        .into_bytes();
        let result: Result<Foo, _> = crate::deserialize_from_slice(&src);
        assert!(result.is_err());
        let path = result.unwrap_err().path.unwrap();
        assert_eq!(path.to_string(), "two.value");
    }

    #[test]
    fn ser() {
        let src = Foo {
            one: Bar { value: 42 },
            two: Bar { value: u64::MAX },
        };
        let result = crate::serialize_to_bson(&src);
        assert!(result.is_err());
        let path = result.unwrap_err().path.unwrap();
        assert_eq!(path.to_string(), "two.value");
    }

    #[test]
    fn ser_raw() {
        let src = Foo {
            one: Bar { value: 42 },
            two: Bar { value: u64::MAX },
        };
        let result = crate::serialize_to_vec(&src);
        assert!(result.is_err());
        let path = result.unwrap_err().path.unwrap();
        assert_eq!(path.to_string(), "two.value");
    }
}
