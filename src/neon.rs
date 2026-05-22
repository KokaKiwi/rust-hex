// SPDX-License-Identifier: Apache-2.0 OR MIT
#![allow(unsafe_code)]

use core::arch::aarch64::{
    uint8x16_t, vandq_u8, vcgeq_u8, vcleq_u8, vgetq_lane_u64, vld1q_dup_u8, vld1q_u8, vorrq_u8,
    vreinterpretq_u64_u8, vsubq_u8,
};
use core::mem;

pub use crate::error::FromHexError;

/// Decode a 64-bit hex value encoded as 16 ASCII characters.
///
/// Returns the decoded value as a `u64`.
///
/// # Safety
///
/// * `addr` must be an address from which at least 16 bytes can be read.
/// * The system must support the ARM Neon extensions.
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
#[allow(unused)]
pub(crate) unsafe fn decode_u64(addr: *const u8) -> Result<u64, FromHexError> {
    // Fill a 128-bit (u8x16) vector register with 16 copies of the specified byte `b`.
    macro_rules! fill {
        ($b:expr) => {
            // SAFETY: This code is nested in a function gated by `target_feature(enable=neon)`.
            unsafe { vld1q_dup_u8(&$b) }
        };
    }

    // Load 16 input bytes from `addr` into a uint8x16_t vector register.
    // SAFETY: The caller guaranteed that `addr` points to at least 16 readable bytes.
    let input_bytes = unsafe { vld1q_u8(addr.cast()) };

    // Compare every input byte with ASCII '0', producing a flag vector in which
    // each element is 0xff if the input byte is >= '0' or 0x00 otherwise.
    let after_start = vcgeq_u8(input_bytes, fill!(b'0'));
    // Now compare every input byte to ASCII '9', producing a flag vector in which
    // every element is 0xff if the input byte is <= '9' or 0x00 otherwise.
    let before_end = vcleq_u8(input_bytes, fill!(b'9'));
    // And those two flag vectors together, producing a flag vector in
    // which each element is 0xff if the corresponding input byte is an
    // ASCII decimal digit or 0x00 otherwise.
    let is_decimal = vandq_u8(after_start, before_end);

    // Use the same technique to produce a flag vector indicating which of
    // the input bytes are uppercase ASCII letters in the range 'A'-'F'.
    let after_start = vcgeq_u8(input_bytes, fill!(b'A'));
    let before_end = vcleq_u8(input_bytes, fill!(b'F'));
    let is_uppercase = vandq_u8(after_start, before_end);

    // Use the same technique to produce a flag vector indicating which of
    // the input bytes are lowercase ASCII letters in the range 'a'-'f'
    let after_start = vcgeq_u8(input_bytes, fill!(b'a'));
    let before_end = vcleq_u8(input_bytes, fill!(b'f'));
    let is_lowercase = vandq_u8(after_start, before_end);

    // If any input byte was _not_ a hex digit, return an error.
    let is_hex_digit = vorrq_u8(is_decimal, vorrq_u8(is_uppercase, is_lowercase));
    let is_hex_digit = vreinterpretq_u64_u8(is_hex_digit);
    let offset = vgetq_lane_u64(is_hex_digit, 0).leading_ones();
    if offset != 64 {
        let offset = 7 - offset as usize / 8;
        // SAFETY: 0 <= offset <= 7, so addr + offset is within the 16-byte block starting at addr.
        let ch = unsafe { *(addr.add(offset)) } as char;
        return Err(FromHexError::InvalidHexCharacter {
            c: ch,
            index: offset,
        });
    }
    let offset = vgetq_lane_u64(is_hex_digit, 1).leading_ones();
    if offset != 64 {
        let offset = 15 - offset as usize / 8;
        // SAFETY: 8 <= offset <= 15, so addr + offset is within the 16-byte block starting at addr.
        let ch = unsafe { *(addr.add(offset)) } as char;
        return Err(FromHexError::InvalidHexCharacter {
            c: ch,
            index: offset,
        });
    }

    // Given the vector of input ASCII characters, subtract the proper amount from each
    // element to convert it to the corresponding hex digit value. The previously computed
    // flag vectors indicate what conversion amount to subtract from each element.
    let mut decoded = vsubq_u8(input_bytes, vandq_u8(is_decimal, fill!(b'0')));
    decoded = vsubq_u8(decoded, vandq_u8(is_uppercase, fill!(b'A' - 10)));
    decoded = vsubq_u8(decoded, vandq_u8(is_lowercase, fill!(b'a' - 10)));

    // The vector now contains 16 hex digits, stored one per byte in big-endian order.
    // Rearrange the bytes so the digits that were at odd-numbered offsets within the
    // vector (which correspond to the lower half of each byte of the data being decoded)
    // end up in the first 8 bytes of the vector, followed by the digits that were at
    // even-numbered offsets (corresponding to the upper half of each byte of data being
    // decoded) in the last 8 bytes.
    // SAFETY: The slice being transmuted has the same memory layout as `uint8x16_t`.
    const SHUFFLE_MAP: uint8x16_t =
        unsafe { mem::transmute([15u8, 13, 11, 9, 7, 5, 3, 1, 14, 12, 10, 8, 6, 4, 2, 0]) };
    let mut shuffled: uint8x16_t;
    // SAFETY: This function is gated by `target_feature(enable = "neon")`.
    unsafe {
        core::arch::asm!(
            "tbl {shuffled:v}.16B, {{ {decoded:v}.16B }}, {map:v}.16B",
            shuffled = out(vreg) shuffled,
            decoded = in(vreg) decoded,
            map = in(vreg) SHUFFLE_MAP,
        )
    };
    // Extract the halves of the 128-bit vector into a pair of 64-bit registers, one
    // containing the upper half of each hex digit of the result and the other containing
    // the lower half. Shift the former left by 4 bits and or it with the latter to get
    // the result.
    let high_low = vreinterpretq_u64_u8(shuffled);
    let high = vgetq_lane_u64(high_low, 1);
    let low = vgetq_lane_u64(high_low, 0);
    Ok((high << 4) | low)
}

// Check whether the Neon SIMD instructions are enabled on the currently running CPU.
#[cfg(target_arch = "aarch64")]
#[allow(unused)]
#[inline]
pub(crate) fn enabled() -> bool {
    #[cfg(feature = "std")]
    return std::arch::is_aarch64_feature_detected!("neon");

    #[cfg(not(feature = "std"))]
    return false;
}

#[cfg(target_arch = "aarch64")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_feature = "neon")]
    fn test_hex_decode_neon() {
        assert_eq!(unsafe { decode_u64("0000000000000000".as_ptr()) }, Ok(0));
        assert_eq!(
            unsafe { decode_u64("fedcba9876543210".as_ptr()) },
            Ok(0xfedcba9876543210)
        );
        assert_eq!(
            unsafe { decode_u64("ffffffffffffffff".as_ptr()) },
            Ok(0xffffffffffffffff)
        );
        assert_eq!(
            unsafe { decode_u64("G123456701234567".as_ptr()) },
            Err(FromHexError::InvalidHexCharacter { c: 'G', index: 0 })
        );
        assert_eq!(
            unsafe { decode_u64("0123456-01234567".as_ptr()) },
            Err(FromHexError::InvalidHexCharacter { c: '-', index: 7 })
        );
        assert_eq!(
            unsafe { decode_u64("01234567!1234567".as_ptr()) },
            Err(FromHexError::InvalidHexCharacter { c: '!', index: 8 })
        );
        assert_eq!(
            unsafe { decode_u64("0123456789abcdeg".as_ptr()) },
            Err(FromHexError::InvalidHexCharacter { c: 'g', index: 15 })
        );
    }
}
