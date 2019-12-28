use core::fmt;

/// The error type for decoding a hex string into `Vec<u8>` or `[u8; N]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FromHexError {
    /// An invalid character was found. Valid ones are: `0...9`, `a...f`
    /// or `A...F`.
    InvalidHexCharacter { c: char, index: usize },

    /// A hex string's length needs to be even, as two digits correspond to
    /// one byte.
    OddLength,

    /// If the hex string is decoded into a fixed sized container, such as an
    /// array, the hex string's length * 2 has to match the container's
    /// length.
    InvalidStringLength,
}

#[cfg(feature = "std")]
impl std::error::Error for FromHexError {
    fn description(&self) -> &str {
        match *self {
            Self::InvalidHexCharacter { .. } => "invalid character",
            Self::OddLength => "odd number of digits",
            Self::InvalidStringLength => "invalid string length",
        }
    }
}

impl fmt::Display for FromHexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidHexCharacter { c, index } => {
                write!(f, "Invalid character '{}' at position {}", c, index)
            }
            Self::OddLength => write!(f, "Odd number of digits"),
            Self::InvalidStringLength => write!(f, "Invalid string length"),
        }
    }
}
