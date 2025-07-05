#![cfg(all(feature = "serde", feature = "heapless"))]
#![allow(clippy::blacklisted_name)]

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Foo<const N: usize> {
    #[serde(
        serialize_with = "hex::serialize_heapless::<_, _, N>",
        deserialize_with = "hex::deserialize"
    )]
    bar: heapless::Vec<u8, N>,
}

#[test]
fn serialize_heapless() {
    let foo: Foo<8> = Foo {
        bar: heapless::Vec::from_slice(&[1, 10, 100, 1]).unwrap(),
    };

    let ser = serde_json_core::to_string::<_, 18>(&foo).expect("serialization failed");
    assert_eq!(ser, r#"{"bar":"010a6401"}"#);
}

#[test]
fn deserialize_heapless() {
    let foo = Foo {
        bar: heapless::Vec::from_slice(&[1, 10, 100, 1]).unwrap(),
    };

    let (de, _): (Foo<8>, usize) =
        serde_json_core::from_str(r#"{"bar":"010a6401"}"#).expect("deserialization failed");
    assert_eq!(de, foo);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Bar<const N: usize> {
    #[serde(
        serialize_with = "hex::serialize_upper_heapless::<_, _, N>",
        deserialize_with = "hex::deserialize"
    )]
    foo: heapless::Vec<u8, N>,
}

#[test]
fn serialize_upper_heapless() {
    let bar: Bar<6> = Bar {
        foo: heapless::Vec::from_slice(&[1, 10, 100]).unwrap(),
    };

    let ser = serde_json_core::to_string::<_, 16>(&bar).expect("serialization failed");
    assert_eq!(ser, r#"{"foo":"010A64"}"#);
}

#[test]
fn deserialize_upper_heapless() {
    let bar = Bar {
        foo: heapless::Vec::from_slice(&[1, 10, 100]).unwrap(),
    };

    let (de, _): (Bar<3>, usize) =
        serde_json_core::from_str(r#"{"foo":"010A64"}"#).expect("deserialization failed");
    assert_eq!(de, bar);
}
