use crate::encoding::json5::Parser;
use crate::encoding::json5::error::{Error, Result};
use crate::encoding::json5::value::{Map, Number, Value};
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};

/// Deserialize directly from a JSON5 string without constructing an intermediate Value.
#[allow(dead_code)]
pub struct Deserializer<'de> {
    // parser: crate::parser::Parser<'de>,
    parser: Parser<'de>,
}

// !TODO undestand for what marked as unused
#[allow(dead_code)]
impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Self { parser: Parser::new(input) }
    }
}

#[allow(unused_macros)]
macro_rules! forward_deserialize_number {
    ($method:ident, $visit:ident, $ty:ty) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let val = self.parser.parse_value()?;
            match val {
                Value::Number(n) => {
                    let v = n.as_f64() as $ty;
                    visitor.$visit(v)
                },
                _ => Err(Error::TypeMismatch { expected: stringify!($ty), got: val.type_name() }),
            }
        }
    };
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        self.parser.skip_whitespace_and_comments();
        let val = self.parser.parse_value()?;
        ValueDeserializer::new(val).deserialize_any(visitor)
    }

    fn deserialize_bool<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        self.parser.skip_whitespace_and_comments();
        match self.parser.parse_value()? {
            Value::Bool(b) => visitor.visit_bool(b),
            v => Err(Error::TypeMismatch { expected: "bool", got: v.type_name() }),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        self.parser.skip_whitespace_and_comments();
        match self.parser.parse_value()? {
            Value::String(s) => visitor.visit_string(s),
            v => Err(Error::TypeMismatch { expected: "str", got: v.type_name() }),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V: Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        self.parser.skip_whitespace_and_comments();
        let val = self.parser.parse_value()?;
        match val {
            Value::Null => visitor.visit_none(),
            other => visitor.visit_some(ValueDeserializer::new(other)),
        }
    }

    serde::forward_to_deserialize_any! {
        i8 i16 i32 i64 i128
        u8 u16 u32 u64 u128
        f32 f64
        char bytes byte_buf
        unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

pub struct ValueDeserializer {
    value: Value,
}

impl ValueDeserializer {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl<'de> de::Deserializer<'de> for ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Number(Number::Int(n)) => visitor.visit_i64(n),
            Value::Number(Number::Uint(n)) => visitor.visit_u64(n),
            Value::Number(Number::Float(f)) => visitor.visit_f64(f),
            Value::Number(Number::NaN) => visitor.visit_f64(f64::NAN),
            Value::Number(Number::Infinity) => visitor.visit_f64(f64::INFINITY),
            Value::Number(Number::NegInfinity) => visitor.visit_f64(f64::NEG_INFINITY),
            Value::String(s) => visitor.visit_string(s),
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a)),
            Value::Object(m) => visitor.visit_map(MapDeserializer::new(m)),
        }
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Bool(b) => visitor.visit_bool(b),
            v => Err(Error::TypeMismatch { expected: "bool", got: v.type_name() }),
        }
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(num_to_int::<i8>(&self.value)?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(num_to_int::<i16>(&self.value)?)
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(num_to_int::<i32>(&self.value)?)
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(num_to_int::<i64>(&self.value)?)
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(num_to_int::<i128>(&self.value)?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(num_to_uint::<u8>(&self.value)?)
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(num_to_uint::<u16>(&self.value)?)
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(num_to_uint::<u32>(&self.value)?)
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(num_to_uint::<u64>(&self.value)?)
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u128(num_to_uint::<u128>(&self.value)?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match &self.value {
            Value::Number(n) => visitor.visit_f32(n.as_f64() as f32),
            v => Err(Error::TypeMismatch { expected: "f32", got: v.type_name() }),
        }
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match &self.value {
            Value::Number(n) => visitor.visit_f64(n.as_f64()),
            v => Err(Error::TypeMismatch { expected: "f64", got: v.type_name() }),
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::String(s) => {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => visitor.visit_char(c),
                    _ => Err(Error::Custom("expected single char".into())),
                }
            },
            v => Err(Error::TypeMismatch { expected: "char", got: v.type_name() }),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::String(s) => visitor.visit_string(s),
            Value::Number(n) => visitor.visit_string(n.to_string()),
            Value::Bool(b) => visitor.visit_string(b.to_string()),
            Value::Null => visitor.visit_string("null".into()),
            v => Err(Error::TypeMismatch { expected: "string", got: v.type_name() }),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::String(s) => visitor.visit_bytes(s.as_bytes()),
            v => Err(Error::TypeMismatch { expected: "bytes", got: v.type_name() }),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::String(s) => visitor.visit_byte_buf(s.into_bytes()),
            v => Err(Error::TypeMismatch { expected: "byte_buf", got: v.type_name() }),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Null => visitor.visit_none(),
            other => visitor.visit_some(ValueDeserializer::new(other)),
        }
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Null => visitor.visit_unit(),
            v => Err(Error::TypeMismatch { expected: "null", got: v.type_name() }),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _name: &'static str, visitor: V) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a)),
            v => Err(Error::TypeMismatch { expected: "array", got: v.type_name() }),
        }
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.value {
            Value::Object(m) => visitor.visit_map(MapDeserializer::new(m)),
            v => Err(Error::TypeMismatch { expected: "object", got: v.type_name() }),
        }
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        match self.value {
            Value::Object(m) => visitor.visit_map(MapDeserializer::new(m)),
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a)),
            v => Err(Error::TypeMismatch { expected: "object", got: v.type_name() }),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        match self.value {
            Value::String(s) => visitor.visit_enum(UnitVariantAccess(s)),
            Value::Object(m) => {
                if m.len() != 1 {
                    return Err(Error::Custom("enum object must have exactly one key".into()));
                }
                let (key, val) = m.into_iter().next().unwrap();
                visitor.visit_enum(EnumDeserializer { variant: key, value: val })
            },
            v => Err(Error::TypeMismatch { expected: "enum", got: v.type_name() }),
        }
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }
}

// -------------------------------------------------------------------------
// Sequence deserializer
// -------------------------------------------------------------------------

struct SeqDeserializer {
    iter: std::vec::IntoIter<Value>,
}

impl SeqDeserializer {
    fn new(v: Vec<Value>) -> Self {
        Self { iter: v.into_iter() }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        match self.iter.next() {
            Some(v) => seed.deserialize(ValueDeserializer::new(v)).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

// -------------------------------------------------------------------------
// Map deserializer
// -------------------------------------------------------------------------

struct MapDeserializer {
    iter: crate::encoding::json5::value::MapIntoIter<String, Value>,
    current_value: Option<Value>,
}

impl MapDeserializer {
    fn new(m: Map<String, Value>) -> Self {
        Self { iter: m.into_iter(), current_value: None }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        match self.iter.next() {
            Some((k, v)) => {
                self.current_value = Some(v);
                seed.deserialize(ValueDeserializer::new(Value::String(k))).map(Some)
            },
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        let v = self.current_value.take().ok_or_else(|| Error::Custom("value called before key".into()))?;
        seed.deserialize(ValueDeserializer::new(v))
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

// -------------------------------------------------------------------------
// Enum deserializers
// -------------------------------------------------------------------------

struct UnitVariantAccess(String);

impl<'de> EnumAccess<'de> for UnitVariantAccess {
    type Error = Error;
    type Variant = UnitOnly;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let v = seed.deserialize(ValueDeserializer::new(Value::String(self.0)))?;
        Ok((v, UnitOnly))
    }
}

struct UnitOnly;

impl<'de> VariantAccess<'de> for UnitOnly {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, _: T) -> Result<T::Value> {
        Err(Error::Custom("expected unit variant".into()))
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, _: V) -> Result<V::Value> {
        Err(Error::Custom("expected unit variant".into()))
    }
    fn struct_variant<V: Visitor<'de>>(self, _: &'static [&'static str], _: V) -> Result<V::Value> {
        Err(Error::Custom("expected unit variant".into()))
    }
}

struct EnumDeserializer {
    variant: String,
    value: Value,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = ContentVariant;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant)> {
        let v = seed.deserialize(ValueDeserializer::new(Value::String(self.variant)))?;
        Ok((v, ContentVariant(self.value)))
    }
}

struct ContentVariant(Value);

impl<'de> VariantAccess<'de> for ContentVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.0 {
            Value::Null => Ok(()),
            _ => Err(Error::Custom("expected null for unit variant".into())),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        seed.deserialize(ValueDeserializer::new(self.0))
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        match self.0 {
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a)),
            v => Err(Error::TypeMismatch { expected: "array", got: v.type_name() }),
        }
    }

    fn struct_variant<V: Visitor<'de>>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value> {
        match self.0 {
            Value::Object(m) => visitor.visit_map(MapDeserializer::new(m)),
            v => Err(Error::TypeMismatch { expected: "object", got: v.type_name() }),
        }
    }
}

// -------------------------------------------------------------------------
// Integer casting helpers
// -------------------------------------------------------------------------

fn num_to_int<T>(val: &Value) -> Result<T>
where
    T: TryFrom<i64> + TryFrom<u64>,
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
{
    match val {
        Value::Number(Number::Int(n)) => T::try_from(*n).map_err(|_| Error::Custom(format!("integer overflow: {}", n))),
        Value::Number(Number::Uint(n)) => {
            T::try_from(*n).map_err(|_| Error::Custom(format!("integer overflow: {}", n)))
        },
        Value::Number(Number::Float(f)) => {
            let n = *f as i64;
            T::try_from(n).map_err(|_| Error::Custom(format!("integer overflow: {}", n)))
        },
        v => Err(Error::TypeMismatch { expected: "integer", got: v.type_name() }),
    }
}

fn num_to_uint<T>(val: &Value) -> Result<T>
where
    T: TryFrom<u64> + TryFrom<i64>,
    <T as TryFrom<u64>>::Error: std::fmt::Debug,
    <T as TryFrom<i64>>::Error: std::fmt::Debug,
{
    match val {
        Value::Number(Number::Uint(n)) => {
            T::try_from(*n).map_err(|_| Error::Custom(format!("integer overflow: {}", n)))
        },
        Value::Number(Number::Int(n)) if *n >= 0 => {
            T::try_from(*n as u64).map_err(|_| Error::Custom(format!("integer overflow: {}", n)))
        },
        Value::Number(Number::Float(f)) if *f >= 0.0 => {
            T::try_from(*f as u64).map_err(|_| Error::Custom(format!("integer overflow: {}", f)))
        },
        v => Err(Error::TypeMismatch { expected: "unsigned int", got: v.type_name() }),
    }
}
