/// JSON5 implementation in Rust with serde support.
/// Spec: https://spec.json5.org/
pub mod de;
pub mod error;
pub mod parser;
pub mod ser;
pub mod value;

pub use error::{Error, Result};
pub use parser::Parser;
#[allow(unused_imports)]
pub use value::{Map, Number, Value};

use serde::{Serialize, de::DeserializeOwned};

/// Deserialize a JSON5 string into a Rust type.
pub fn from_str<T: DeserializeOwned>(input: &str) -> Result<T> {
    let value = parse_value(input)?;
    T::deserialize(de::ValueDeserializer::new(value))
}

/// Serialize a Rust type into a JSON5 string.
#[allow(dead_code)]
pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    ser::serialize(value)
}

/// Serialize with pretty-printing (indented).
pub fn to_string_pretty<T: Serialize>(value: &T) -> Result<String> {
    ser::serialize_with_formatter(value, &mut ser::PrettyFormatter::new("    ", false))
}

/// Parse a JSON5 string into a `Value`.
pub fn parse_value(input: &str) -> Result<Value> {
    let mut parser = Parser::new(input);
    let val = parser.parse_value()?;
    parser.skip_whitespace_and_comments();
    if parser.remaining() > 0 {
        return Err(Error::TrailingData(parser.pos()));
    }
    Ok(val)
}

#[cfg(test)]
#[cfg(not(clippy))]
mod tests;
