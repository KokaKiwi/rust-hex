// Copyright (c) 2013-2014 The Rust Project Developers.
// Copyright (c) 2015-2021 The rust-hex Developers.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Encoding and decoding hex strings.
//!
//! For most cases, you can simply use the [`decode`], [`encode`] and
//! [`encode_upper`] functions. If you need a bit more control, use the traits
//! [`ToHex`] and [`FromHex`] instead.
//!
//! # Example
//!
//! ```
//! # #[cfg(not(feature = "alloc"))]
//! # let mut output = [0; 0x18];
//! #
//! # #[cfg(not(feature = "alloc"))]
//! # hex::encode_to_slice(b"Hello world!", &mut output).unwrap();
//! #
//! # #[cfg(not(feature = "alloc"))]
//! # let hex_string = ::core::str::from_utf8(&output).unwrap();
//! #
//! # #[cfg(feature = "alloc")]
//! let hex_string = hex::encode("Hello world!");
//!
//! println!("{}", hex_string); // Prints "48656c6c6f20776f726c6421"
//!
//! # assert_eq!(hex_string, "48656c6c6f20776f726c6421");
//! ```

#![doc(html_root_url = "https://docs.rs/hex/0.4.3")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::unreadable_literal)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use core::iter;

mod error;
pub use crate::error::FromHexError;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod serde;
#[cfg(feature = "serde")]
pub use crate::serde::deserialize;
#[cfg(all(feature = "alloc", feature = "serde"))]
pub use crate::serde::{serialize, serialize_upper};

/// Encoding values as hex string.
///
/// This trait is implemented for all `T` which implement `AsRef<[u8]>`. This
/// includes `String`, `str`, `Vec<u8>` and `[u8]`.
///
/// # Example
///
/// ```
/// use hex::ToHex;
///
/// println!("{}", "Hello world!".encode_hex::<String>());
/// # assert_eq!("Hello world!".encode_hex::<String>(), "48656c6c6f20776f726c6421".to_string());
/// ```
///
/// *Note*: instead of using this trait, you might want to use [`encode()`].
pub trait ToHex {
    /// Encode the hex strict representing `self` into the result. Lower case
    /// letters are used (e.g. `f9b4ca`)
    fn encode_hex<T: iter::FromIterator<char>>(&self) -> T;

    /// Encode the hex strict representing `self` into the result. Upper case
    /// letters are used (e.g. `F9B4CA`)
    fn encode_hex_upper<T: iter::FromIterator<char>>(&self) -> T;
}

const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";
const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

struct BytesToHexChars<'a> {
    inner: ::core::slice::Iter<'a, u8>,
    table: &'static [u8; 16],
    next: Option<char>,
}

impl<'a> BytesToHexChars<'a> {
    fn new(inner: &'a [u8], table: &'static [u8; 16]) -> BytesToHexChars<'a> {
        BytesToHexChars {
            inner: inner.iter(),
            table,
            next: None,
        }
    }
}

impl Iterator for BytesToHexChars<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next.take() {
            Some(current) => Some(current),
            None => self.inner.next().map(|byte| {
                let current = self.table[(byte >> 4) as usize] as char;
                self.next = Some(self.table[(byte & 0x0F) as usize] as char);
                current
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.len();
        (length, Some(length))
    }
}

impl iter::ExactSizeIterator for BytesToHexChars<'_> {
    fn len(&self) -> usize {
        let mut length = self.inner.len() * 2;
        if self.next.is_some() {
            length += 1;
        }
        length
    }
}

#[inline]
fn encode_to_iter<T: iter::FromIterator<char>>(table: &'static [u8; 16], source: &[u8]) -> T {
    BytesToHexChars::new(source, table).collect()
}

impl<T: AsRef<[u8]>> ToHex for T {
    fn encode_hex<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(HEX_CHARS_LOWER, self.as_ref())
    }

    fn encode_hex_upper<U: iter::FromIterator<char>>(&self) -> U {
        encode_to_iter(HEX_CHARS_UPPER, self.as_ref())
    }
}

/// Types that can be decoded from a hex string.
///
/// This trait is implemented for `Vec<u8>` and small `u8`-arrays.
///
/// # Example
///
/// ```
/// use core::str;
/// use hex::FromHex;
///
/// let buffer = <[u8; 12]>::from_hex("48656c6c6f20776f726c6421")?;
/// let string = str::from_utf8(&buffer).expect("invalid buffer length");
///
/// println!("{}", string); // prints "Hello world!"
/// # assert_eq!("Hello world!", string);
/// # Ok::<(), hex::FromHexError>(())
/// ```
pub trait FromHex: Sized {
    type Error;

    /// Creates an instance of type `Self` from the given hex string, or fails
    /// with a custom error type.
    ///
    /// Both, upper and lower case characters are valid and can even be
    /// mixed (e.g. `f9b4ca`, `F9B4CA` and `f9B4Ca` are all valid strings).
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error>;
}

const fn val(c: u8, idx: usize) -> Result<u8, FromHexError> {
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

#[cfg(feature = "alloc")]
impl FromHex for Vec<u8> {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let hex = hex.as_ref();
        if hex.len() % 2 != 0 {
            return Err(FromHexError::OddLength);
        }

        hex.chunks(2)
            .enumerate()
            .map(|(i, pair)| Ok(val(pair[0], 2 * i)? << 4 | val(pair[1], 2 * i + 1)?))
            .collect()
    }
}

impl<const N: usize> FromHex for [u8; N] {
    type Error = FromHexError;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        let mut out = [0_u8; N];
        decode_to_slice(hex, &mut out as &mut [u8])?;

        Ok(out)
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
#[must_use]
#[cfg(feature = "alloc")]
pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    data.encode_hex()
}

/// Encodes `data` as hex string using uppercase characters.
///
/// Apart from the characters' casing, this works exactly like `encode()`.
///
/// # Example
///
/// ```
/// assert_eq!(hex::encode_upper("Hello world!"), "48656C6C6F20776F726C6421");
/// assert_eq!(hex::encode_upper(vec![1, 2, 3, 15, 16]), "0102030F10");
/// ```
#[must_use]
#[cfg(feature = "alloc")]
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
#[cfg(feature = "alloc")]
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
#[must_use]
const fn byte2hex(byte: u8, table: &[u8; 16]) -> (u8, u8) {
    let high = table[((byte & 0xf0) >> 4) as usize];
    let low = table[(byte & 0x0f) as usize];

    (high, low)
}

/// Encodes some bytes into a mutable slice of bytes.
///
/// The output buffer, has to be able to hold exactly `input.len() * 2` bytes,
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
///
/// If the buffer is too large, an error is returned:
///
/// ```
/// use hex::FromHexError;
/// # fn main() -> Result<(), FromHexError> {
/// let mut bytes = [0_u8; 5 * 2];
///
/// assert_eq!(hex::encode_to_slice(b"kiwi", &mut bytes), Err(FromHexError::InvalidStringLength));
///
/// // you can do this instead:
/// hex::encode_to_slice(b"kiwi", &mut bytes[..4 * 2])?;
/// assert_eq!(&bytes, b"6b697769\0\0");
/// # Ok(())
/// # }
/// ```
pub fn encode_to_slice<T: AsRef<[u8]>>(input: T, output: &mut [u8]) -> Result<(), FromHexError> {
    if input.as_ref().len() * 2 != output.len() {
        return Err(FromHexError::InvalidStringLength);
    }

    for (byte, (i, j)) in input
        .as_ref()
        .iter()
        .zip(generate_iter(input.as_ref().len() * 2))
    {
        let (high, low) = byte2hex(*byte, HEX_CHARS_LOWER);
        output[i] = high;
        output[j] = low;
    }

    Ok(())
}

/// A wrapper around binary data which formats them as hex when [displaying
/// it][core::fmt::Display].
///
/// See [`display`] function.
pub struct Display<'a>(&'a [u8]);

impl Display<'_> {
    fn do_fmt(&self, fmtr: &mut core::fmt::Formatter<'_>, chars: &[u8; 16]) -> core::fmt::Result {
        debug_assert!(chars.is_ascii());
        let mut buffer = [core::mem::MaybeUninit::<u8>::uninit(); 512];
        for chunk in self.0.chunks(buffer.len() / 2) {
            // TODO: Use `array_chunks_mut` instead of `chunks_exact_mut` once
            // it stabilises.
            for (out, &byte) in buffer.chunks_exact_mut(2).zip(chunk.iter()) {
                let (high, low) = byte2hex(byte, chars);
                out[0].write(high);
                out[1].write(low);
            }
            let len = chunk.len() * 2;
            let chunk = (&buffer[..len]) as *const [_] as *const [u8];
            // SAFETY: we've just filled the buffer up to len with hex digits
            // which are ASCII characters.
            fmtr.write_str(unsafe { core::str::from_utf8_unchecked(&*chunk) })?;
        }
        Ok(())
    }
}

impl core::fmt::Display for Display<'_> {
    #[inline]
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.do_fmt(fmtr, HEX_CHARS_LOWER)
    }
}

impl core::fmt::LowerHex for Display<'_> {
    #[inline]
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.do_fmt(fmtr, HEX_CHARS_LOWER)
    }
}

impl core::fmt::UpperHex for Display<'_> {
    #[inline]
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.do_fmt(fmtr, HEX_CHARS_UPPER)
    }
}

impl core::fmt::Debug for Display<'_> {
    #[inline]
    fn fmt(&self, fmtr: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.do_fmt(fmtr, HEX_CHARS_LOWER)
    }
}

/// Wraps the value in an object which can be formatted as hex.
///
/// The benefit over using [`encode`] is that no memory allocations are done
/// when formatting the value.
///
/// # Example
///
/// ```
/// assert_eq!(hex::display(b"kiwi").to_string(), "6b697769");
/// assert_eq!(format!("{}", hex::display(b"kiwi")), "6b697769");
/// assert_eq!(format!("{:X}", hex::display(b"kiwi")), "6B697769");
/// ```
#[inline]
pub fn display<T: AsRef<[u8]>>(input: &T) -> Display<'_> {
    Display(input.as_ref())
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_gen_iter() {
        let result = alloc::vec![(0, 1), (2, 3)];

        assert_eq!(generate_iter(5).collect::<Vec<_>>(), result);
    }

    const TEST_CASES: [(&[u8], &str, &str); 5] = [
        (b"", "", ""),
        (b"kiwi", "6b697769", "6B697769"),
        (b"kiwis", "6b69776973", "6B69776973"),
        (b"foobar", "666f6f626172", "666F6F626172"),
        (b"\xef\xbb\xbf", "efbbbf", "EFBBBF"),
    ];

    #[test]
    fn test_encode_to_slice() {
        let mut buffer = [0; 16];

        for (bytes, lower, _upper) in TEST_CASES {
            buffer.fill(0);
            let output = &mut buffer[..lower.len()];
            encode_to_slice(bytes, output).unwrap();
            assert_eq!(output, lower.as_bytes());
        }

        assert_eq!(
            encode_to_slice(b"kiwis", &mut buffer),
            Err(FromHexError::InvalidStringLength)
        );
    }

    #[test]
    fn test_decode_to_slice() {
        let mut buffer = [0; 8];

        for (bytes, lower, upper) in TEST_CASES {
            let output = &mut buffer[..bytes.len()];
            decode_to_slice(lower, output).unwrap();
            assert_eq!(output, bytes);
            decode_to_slice(upper, output).unwrap();
            assert_eq!(output, bytes);
        }

        assert_eq!(
            decode_to_slice(b"kiwi", &mut buffer),
            Err(FromHexError::InvalidStringLength)
        );
        assert_eq!(
            decode_to_slice(b"kiwis", &mut buffer),
            Err(FromHexError::OddLength)
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_encode() {
        for (bytes, lower, upper) in TEST_CASES {
            assert_eq!(encode(bytes), lower);
            assert_eq!(encode_upper(bytes), upper);
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_to_hex() {
        for (bytes, lower, upper) in TEST_CASES {
            assert_eq!(bytes.encode_hex::<String>(), lower);
            assert_eq!(bytes.encode_hex_upper::<String>(), upper);
        }
    }

    #[test]
    fn test_display() {
        for (bytes, lower, upper) in TEST_CASES {
            let disp = display(&bytes);
            assert_eq!(alloc::format!("{disp}"), lower);
            assert_eq!(alloc::format!("{disp:?}"), lower);
            assert_eq!(alloc::format!("{disp:x}"), lower);
            assert_eq!(alloc::format!("{disp:X}"), upper);
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_decode() {
        for (bytes, lower, upper) in TEST_CASES {
            assert_eq!(decode(lower).unwrap(), bytes);
            assert_eq!(decode(lower.as_bytes()).unwrap(), bytes);
            assert_eq!(decode(upper).unwrap(), bytes);
            assert_eq!(decode(upper.as_bytes()).unwrap(), bytes);
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_from_hex() {
        for (bytes, lower, upper) in TEST_CASES {
            assert_eq!(Vec::from_hex(lower).unwrap(), bytes);
            assert_eq!(Vec::from_hex(lower.as_bytes()).unwrap(), bytes);
            assert_eq!(Vec::from_hex(upper).unwrap(), bytes);
            assert_eq!(Vec::from_hex(upper.as_bytes()).unwrap(), bytes);
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_length() {
        assert_eq!(Vec::from_hex("1").unwrap_err(), FromHexError::OddLength);
        assert_eq!(
            Vec::from_hex("666f6f6261721").unwrap_err(),
            FromHexError::OddLength
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    pub fn test_invalid_char() {
        assert_eq!(
            Vec::from_hex("66ag").unwrap_err(),
            FromHexError::InvalidHexCharacter { c: 'g', index: 3 }
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
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
