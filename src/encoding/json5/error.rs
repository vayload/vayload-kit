use serde::{de, ser};
use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Unexpected character at position
    UnexpectedChar(char, usize),
    /// Unexpected end of input
    UnexpectedEof,
    /// Invalid escape sequence
    InvalidEscape(char),
    /// Invalid unicode escape
    InvalidUnicode(u32),
    /// Invalid number
    InvalidNumber(String),
    /// Trailing data after valid JSON5
    TrailingData(usize),
    /// Duplicate key in object
    #[allow(unused)]
    DuplicateKey(String),
    /// Expected specific character
    Expected(char, Option<char>),
    /// Custom serde error
    Custom(String),
    /// Type mismatch during deserialization
    TypeMismatch { expected: &'static str, got: &'static str },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedChar(c, pos) => write!(f, "Unexpected char {:?} at pos {}", c, pos),
            Error::UnexpectedEof => write!(f, "Unexpected end of input"),
            Error::InvalidEscape(c) => write!(f, "Invalid escape sequence: \\{}", c),
            Error::InvalidUnicode(n) => write!(f, "Invalid unicode code point: U+{:04X}", n),
            Error::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            Error::TrailingData(pos) => write!(f, "Trailing data at position {}", pos),
            Error::DuplicateKey(k) => write!(f, "Duplicate key: {:?}", k),
            Error::Expected(c, got) => write!(f, "Expected {:?}, got {:?}", c, got),
            Error::Custom(s) => write!(f, "{}", s),
            Error::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, got)
            },
        }
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
