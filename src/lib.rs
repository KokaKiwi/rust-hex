// Copyright (c) 2013-2014 The Rust Project Developers.
// Copyright (c) 2015-2018 The rust-hex Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//! Encoding and decoding hex strings.
//!
//! For most cases, you can simply use the [`decode`], [`encode`] and
//! [`encode_upper`] functions. If you need a bit more control, use the traits
//! [`ToHex`] and [`FromHex`] instead.
//!
//! # Example
//!
//! ```
//! fn main() {
//!     let hex_string = hex::encode("Hello world!");
//!     println!("{}", hex_string); // Prints '48656c6c6f20776f726c6421'
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unreadable_literal)]
#![warn(clippy::use_self)]

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use core::iter;

mod error;
pub use error::FromHexError;
mod traits;
pub use traits::{FromHex, ToHex};

const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";
const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

struct BytesToHexChars<'a> {
    inner: ::core::slice::Iter<'a, u8>,
    table: &'static [u8; 16],
    next: Option<char>,
}

impl<'a> BytesToHexChars<'a> {
    fn new(inner: &'a [u8], table: &'static [u8; 16]) -> BytesToHexChars<'a> {
        Self {
            inner: inner.iter(),
            table,
            next: None,
        }
    }
}

impl<'a> Iterator for BytesToHexChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let current = self.table[(byte >> 4) as usize] as char;
                self.next = Some(self.table[(byte & 0xf) as usize] as char);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl<'a> iter::ExactSizeIterator for BytesToHexChars<'a> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

fn encode_to_iter<T: iter::FromIterator<char>>(table: &'static [u8; 16], source: &[u8]) -> T {
    BytesToHexChars::new(source, table).collect()
}

fn val(c: u8, idx: usize) -> Result<u8, FromHexError> {
    match c {
        b'A'..=b'F' => Ok(c - b'A' + 10),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'0'..=b'9' => Ok(c - b'0'),
        _ => Err(FromHexError::InvalidHexCharacter {
            c: c as char,
            index: idx,
        }),
    }
}

/// Encodes `data` as hex string using lowercase characters.
///
/// Lowercase characters are used (e.g. `f9b4ca`). The resulting string's
/// length is always even, each byte in `data` is always encoded using two hex
/// digits. Thus, the resulting string contains exactly twice as many bytes as
/// the input data.
///
/// # Example
///
/// ```
/// assert_eq!(hex::encode("Hello world!"), "48656c6c6f20776f726c6421");
/// assert_eq!(hex::encode(vec![1, 2, 3, 15, 16]), "0102030f10");
/// ```
pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    data.encode_hex()
}

// generates an iterator like this
// (0, 1)
// (2, 3)
// (4, 5)
// (6, 7)
// ...
#[inline]
fn generate_iter(len: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..len).step_by(2).zip((0..len).skip(1).step_by(2))
}

// the inverse of `val`.
#[inline]
fn byte2hex(byte: u8, table: &[u8; 16]) -> (u8, u8) {
    let high = table[((byte & 0xf0) >> 4) as usize];
    let low = table[(byte & 0x0f) as usize];

    (high, low)
}

/// Encodes some bytes into a mutable slice of bytes.
///
/// The output buffer, has to be able to hold at least `input.len() * 2` bytes,
/// otherwise this function will return an error.
///
/// # Example
///
/// ```
/// # use hex::FromHexError;
/// # fn main() -> Result<(), FromHexError> {
/// let mut bytes = [0u8; 4 * 2];
///
/// hex::encode_to_slice(b"kiwi", &mut bytes)?;
/// assert_eq!(&bytes, b"6b697769");
/// # Ok(())
/// # }
/// ```
pub fn encode_to_slice(input: &[u8], output: &mut [u8]) -> Result<(), FromHexError> {
    if input.len() * 2 != output.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    for (byte, (i, j)) in input.iter().zip(generate_iter(input.len() * 2)) {
        let (high, low) = byte2hex(*byte, HEX_CHARS_LOWER);
        output[i] = high;
        output[j] = low;
    }

    Ok(())
}

/// Encodes `data` as hex string using uppercase characters.
///
/// Apart from the characters' casing, this works exactly like [`encode`].
///
/// # Example
/// ```
/// assert_eq!(hex::encode_upper("Hello world!"), "48656C6C6F20776F726C6421");
/// assert_eq!(hex::encode_upper(vec![1, 2, 3, 15, 16]), "0102030F10");
/// ```
pub fn encode_upper<T: AsRef<[u8]>>(data: T) -> String {
    data.encode_hex_upper()
}

/// Decodes a hex string into raw bytes.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
///
/// # Example
///
/// ```
/// assert_eq!(
///     hex::decode("48656c6c6f20776f726c6421"),
///     Ok("Hello world!".to_owned().into_bytes())
/// );
///
/// assert_eq!(hex::decode("123"), Err(hex::FromHexError::OddLength));
/// assert!(hex::decode("foo").is_err());
/// ```
pub fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, FromHexError> {
    FromHex::from_hex(data)
}

/// Decode a hex string into a mutable bytes slice.
///
/// Both, upper and lower case characters are valid in the input string and can
/// even be mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
///
/// # Example
///
/// ```
/// let mut bytes = [0u8; 4];
/// assert_eq!(hex::decode_to_slice("6b697769", &mut bytes as &mut [u8]), Ok(()));
/// assert_eq!(&bytes, b"kiwi");
/// ```
pub fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), FromHexError> {
    let data = data.as_ref();

    if data.len() % 2 != 0 {
        return Err(FromHexError::OddLength);
    }
    if data.len() / 2 != out.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    for (i, byte) in out.iter_mut().enumerate() {
        *byte = val(data[2 * i], 2 * i)? << 4 | val(data[2 * i + 1], 2 * i + 1)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::traits::{FromHex, ToHex};

    #[test]
    fn test_gen_iter() {
        let mut result = Vec::new();
        result.push((0, 1));
        result.push((2, 3));

        assert_eq!(generate_iter(5).collect::<Vec<_>>(), result);
    }

    #[test]
    fn test_encode_to_slice() {
        let mut output_1 = [0; 4 * 2];
        encode_to_slice(b"kiwi", &mut output_1).unwrap();
        assert_eq!(&output_1, b"6b697769");

        let mut output_2 = [0; 5 * 2];
        encode_to_slice(b"kiwis", &mut output_2).unwrap();
        assert_eq!(&output_2, b"6b69776973")
    }

    #[test]
    fn test_encode() {
        assert_eq!(encode("foobar"), "666f6f626172");
    }

    #[test]
    fn test_decode() {
        assert_eq!(
            decode("666f6f626172"),
            Ok(String::from("foobar").into_bytes())
        );
    }

    #[test]
    pub fn test_from_hex_okay_str() {
        assert_eq!(Vec::from_hex("666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex("666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    pub fn test_from_hex_okay_bytes() {
        assert_eq!(Vec::from_hex(b"666f6f626172").unwrap(), b"foobar");
        assert_eq!(Vec::from_hex(b"666F6F626172").unwrap(), b"foobar");
    }

    #[test]
    pub fn test_invalid_length() {
        assert_eq!(Vec::from_hex("1").unwrap_err(), FromHexError::OddLength);
        assert_eq!(
            Vec::from_hex("666f6f6261721").unwrap_err(),
            FromHexError::OddLength
        );
    }

    #[test]
    pub fn test_invalid_char() {
        assert_eq!(
            Vec::from_hex("66ag").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }

    #[test]
    pub fn test_empty() {
        assert_eq!(Vec::from_hex("").unwrap(), b"");
    }

    #[test]
    pub fn test_from_hex_whitespace() {
        assert_eq!(
            Vec::from_hex("666f 6f62617").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: ' ', index: 4 }
        );
    }

    #[test]
    pub fn test_from_hex_array() {
        assert_eq!(
            <[u8; 6] as FromHex>::from_hex("666f6f626172"),
            Ok([0x66, 0x6f, 0x6f, 0x62, 0x61, 0x72])
        );

        assert_eq!(
            <[u8; 5] as FromHex>::from_hex("666f6f626172"),
            Err(FromHexError::InvalidStringLength)
        );
    }
}
