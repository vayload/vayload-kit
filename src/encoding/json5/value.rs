use std::fmt;

use indexmap::{IndexMap, map::IntoIter as IndexMapIntoIter};
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer};

pub type Map<K, V> = IndexMap<K, V>;
pub type MapIntoIter<K, V> = IndexMapIntoIter<K, V>;

/// JSON5 number types — extends JSON with NaN, Infinity, hex literals
#[derive(Clone, Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Uint(u64),
    Float(f64),
    /// JSON5: NaN
    NaN,
    /// JSON5: Infinity
    Infinity,
    /// JSON5: -Infinity
    NegInfinity,
}

impl Number {
    pub fn as_f64(&self) -> f64 {
        match self {
            Number::Int(n) => *n as f64,
            Number::Uint(n) => *n as f64,
            Number::Float(f) => *f,
            Number::NaN => f64::NAN,
            Number::Infinity => f64::INFINITY,
            Number::NegInfinity => f64::NEG_INFINITY,
        }
    }

    // pub fn as_i64(&self) -> Option<i64> {
    //     match self {
    //         Number::Int(n) => Some(*n),
    //         Number::Uint(n) => i64::try_from(*n).ok(),
    //         Number::Float(f) if f.fract() == 0.0 => Some(*f as i64),
    //         _ => None,
    //     }
    // }

    // pub fn as_u64(&self) -> Option<u64> {
    //     match self {
    //         Number::Uint(n) => Some(*n),
    //         Number::Int(n) if *n >= 0 => Some(*n as u64),
    //         Number::Float(f) if f.fract() == 0.0 && *f >= 0.0 => Some(*f as u64),
    //         _ => None,
    //     }
    // }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Int(n) => write!(f, "{}", n),
            Number::Uint(n) => write!(f, "{}", n),
            Number::Float(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{:.1}", n)
                } else {
                    write!(f, "{}", n)
                }
            },
            Number::NaN => write!(f, "NaN"),
            Number::Infinity => write!(f, "Infinity"),
            Number::NegInfinity => write!(f, "-Infinity"),
        }
    }
}

/// JSON5 value — superset of JSON
#[derive(Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "Null"),
            Value::Bool(b) => write!(f, "Bool({})", b),
            Value::Number(n) => write!(f, "Number({})", n),
            Value::String(s) => write!(f, "String({:?})", s),
            Value::Array(a) => write!(f, "Array({:?})", a),
            Value::Object(o) => write!(f, "Object({:?})", o),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            },
            Value::Object(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}: {}", k, v)?;
                }
                write!(f, "}}")
            },
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(Number::Int(n))
    }
}
impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Number(Number::Uint(n))
    }
}
impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(Number::Float(f))
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_owned())
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(n) => match n {
                Number::Int(i) => serializer.serialize_i64(*i),
                Number::Uint(u) => serializer.serialize_u64(*u),
                Number::Float(f) => serializer.serialize_f64(*f),
                Number::NaN => serializer.serialize_f64(f64::NAN),
                Number::Infinity => serializer.serialize_f64(f64::INFINITY),
                Number::NegInfinity => serializer.serialize_f64(f64::NEG_INFINITY),
            },
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for v in arr {
                    seq.serialize_element(v)?;
                }
                seq.end()
            },
            Value::Object(map) => {
                let mut map_ser = serializer.serialize_map(Some(map.len()))?;
                for (k, v) in map {
                    map_ser.serialize_entry(k, v)?;
                }
                map_ser.end()
            },
        }
    }
}
