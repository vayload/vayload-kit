use crate::encoding::json5::error::{Error, Result};
use crate::encoding::json5::value::{Map, Number, Value};
use serde::{Serialize, ser};

pub struct ValueSerializer;

impl ser::Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    fn serialize_bool(self, v: bool) -> Result<Value> {
        Ok(Value::Bool(v))
    }
    fn serialize_i8(self, v: i8) -> Result<Value> {
        Ok(Value::Number(Number::Int(v as i64)))
    }
    fn serialize_i16(self, v: i16) -> Result<Value> {
        Ok(Value::Number(Number::Int(v as i64)))
    }
    fn serialize_i32(self, v: i32) -> Result<Value> {
        Ok(Value::Number(Number::Int(v as i64)))
    }
    fn serialize_i64(self, v: i64) -> Result<Value> {
        Ok(Value::Number(Number::Int(v)))
    }
    fn serialize_i128(self, v: i128) -> Result<Value> {
        Ok(Value::Number(Number::Float(v as f64)))
    }
    fn serialize_u8(self, v: u8) -> Result<Value> {
        Ok(Value::Number(Number::Uint(v as u64)))
    }
    fn serialize_u16(self, v: u16) -> Result<Value> {
        Ok(Value::Number(Number::Uint(v as u64)))
    }
    fn serialize_u32(self, v: u32) -> Result<Value> {
        Ok(Value::Number(Number::Uint(v as u64)))
    }
    fn serialize_u64(self, v: u64) -> Result<Value> {
        Ok(Value::Number(Number::Uint(v)))
    }
    fn serialize_u128(self, v: u128) -> Result<Value> {
        Ok(Value::Number(Number::Float(v as f64)))
    }
    fn serialize_f32(self, v: f32) -> Result<Value> {
        self.serialize_f64(v as f64)
    }
    fn serialize_f64(self, v: f64) -> Result<Value> {
        Ok(Value::Number(if v.is_nan() {
            Number::NaN
        } else if v.is_infinite() {
            if v > 0.0 { Number::Infinity } else { Number::NegInfinity }
        } else {
            Number::Float(v)
        }))
    }
    fn serialize_char(self, v: char) -> Result<Value> {
        Ok(Value::String(v.to_string()))
    }
    fn serialize_str(self, v: &str) -> Result<Value> {
        Ok(Value::String(v.to_owned()))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Value> {
        let arr: Vec<Value> = v.iter().map(|&b| Value::Number(Number::Uint(b as u64))).collect();
        Ok(Value::Array(arr))
    }
    fn serialize_none(self) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Value> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        Ok(Value::Null)
    }
    fn serialize_unit_variant(self, _name: &'static str, _idx: u32, variant: &'static str) -> Result<Value> {
        Ok(Value::String(variant.to_owned()))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self, _name: &'static str, value: &T) -> Result<Value> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value> {
        let mut map = Map::new();
        map.insert(variant.to_owned(), value.serialize(ValueSerializer)?);
        Ok(Value::Object(map))
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<SeqSerializer> {
        Ok(SeqSerializer { arr: Vec::with_capacity(len.unwrap_or(4)) })
    }
    fn serialize_tuple(self, len: usize) -> Result<SeqSerializer> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<SeqSerializer> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<TupleVariantSerializer> {
        Ok(TupleVariantSerializer { variant: variant.to_owned(), arr: Vec::with_capacity(len) })
    }
    fn serialize_map(self, len: Option<usize>) -> Result<MapSerializer> {
        Ok(MapSerializer { map: Map::new(), pending_key: None, _cap: len.unwrap_or(4) })
    }
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<MapSerializer> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _idx: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<StructVariantSerializer> {
        Ok(StructVariantSerializer { variant: variant.to_owned(), map: Map::new() })
    }
}

pub struct SeqSerializer {
    arr: Vec<Value>,
}
impl ser::SerializeSeq for SeqSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.arr.push(v.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.arr))
    }
}
impl ser::SerializeTuple for SeqSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}
impl ser::SerializeTupleStruct for SeqSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, v)
    }
    fn end(self) -> Result<Value> {
        ser::SerializeSeq::end(self)
    }
}

pub struct TupleVariantSerializer {
    variant: String,
    arr: Vec<Value>,
}
impl ser::SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        self.arr.push(v.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value> {
        let mut m = Map::new();
        m.insert(self.variant, Value::Array(self.arr));
        Ok(Value::Object(m))
    }
}

pub struct MapSerializer {
    map: Map<String, Value>,
    pending_key: Option<String>,
    _cap: usize,
}
impl ser::SerializeMap for MapSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, k: &T) -> Result<()> {
        let key_val = k.serialize(ValueSerializer)?;
        let key = match key_val {
            Value::String(s) => s,
            other => other.to_string(),
        };
        self.pending_key = Some(key);
        Ok(())
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, v: &T) -> Result<()> {
        let k = self.pending_key.take().ok_or_else(|| Error::Custom("value without key".into()))?;
        self.map.insert(k, v.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}
impl ser::SerializeStruct for MapSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, k: &'static str, v: &T) -> Result<()> {
        self.map.insert(k.to_owned(), v.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.map))
    }
}

pub struct StructVariantSerializer {
    variant: String,
    map: Map<String, Value>,
}
impl ser::SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = Error;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, k: &'static str, v: &T) -> Result<()> {
        self.map.insert(k.to_owned(), v.serialize(ValueSerializer)?);
        Ok(())
    }
    fn end(self) -> Result<Value> {
        let mut outer = Map::new();
        outer.insert(self.variant, Value::Object(self.map));
        Ok(Value::Object(outer))
    }
}

/// Maximum depth for JSON serialization.
const MAX_DEPTH: usize = 512;

pub trait Formatter {
    fn write_null(&mut self, out: &mut String) -> Result<()>;
    fn write_bool(&mut self, out: &mut String, v: bool) -> Result<()>;
    fn write_number(&mut self, out: &mut String, n: &Number) -> Result<()>;
    fn write_string(&mut self, out: &mut String, s: &str) -> Result<()>;
    fn write_array(&mut self, out: &mut String, arr: &[Value], depth: usize) -> Result<()>;
    fn write_object(&mut self, out: &mut String, obj: &Map<String, Value>, depth: usize) -> Result<()>;
    fn write_value(&mut self, out: &mut String, v: &Value, depth: usize) -> Result<()>;
    fn write_object_key(&mut self, out: &mut String, k: &str) -> Result<()>;
}

pub struct CompactFormatter {
    pub quote_keys: bool,
    max_depth: usize,
}

impl CompactFormatter {
    pub fn new(quote_keys: bool, max_depth: Option<usize>) -> Self {
        Self { quote_keys, max_depth: max_depth.unwrap_or(MAX_DEPTH) }
    }
}

impl Formatter for CompactFormatter {
    fn write_null(&mut self, out: &mut String) -> Result<()> {
        out.push_str("null");
        Ok(())
    }

    fn write_bool(&mut self, out: &mut String, v: bool) -> Result<()> {
        out.push_str(if v { "true" } else { "false" });
        Ok(())
    }

    fn write_number(&mut self, out: &mut String, n: &Number) -> Result<()> {
        out.push_str(&n.to_string());
        Ok(())
    }

    fn write_string(&mut self, out: &mut String, s: &str) -> Result<()> {
        write_escaped_str(out, s, true);
        Ok(())
    }

    fn write_value(&mut self, out: &mut String, v: &Value, depth: usize) -> Result<()> {
        if depth > self.max_depth {
            return Err(Error::Custom("Recursion limit exceeded".into()));
        }
        match v {
            Value::Null => self.write_null(out),
            Value::Bool(b) => self.write_bool(out, *b),
            Value::Number(n) => self.write_number(out, n),
            Value::String(s) => self.write_string(out, s),
            Value::Array(arr) => self.write_array(out, arr, depth),
            Value::Object(map) => self.write_object(out, map, depth),
        }
    }

    fn write_array(&mut self, out: &mut String, arr: &[Value], depth: usize) -> Result<()> {
        out.push('[');
        for (i, v) in arr.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            self.write_value(out, v, depth + 1)?;
        }
        out.push(']');
        Ok(())
    }

    fn write_object(&mut self, out: &mut String, obj: &Map<String, Value>, depth: usize) -> Result<()> {
        out.push('{');
        for (i, (k, v)) in obj.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            self.write_object_key(out, k)?;
            out.push(':');
            self.write_value(out, v, depth + 1)?;
        }
        out.push('}');
        Ok(())
    }

    fn write_object_key(&mut self, out: &mut String, k: &str) -> Result<()> {
        if !self.quote_keys && is_valid_identifier(k) {
            out.push_str(k);
        } else {
            write_escaped_str(out, k, true);
        }
        Ok(())
    }
}

pub struct PrettyFormatter<'a> {
    indent_str: &'a str,
    pub quote_keys: bool,
}

impl<'a> PrettyFormatter<'a> {
    pub fn new(indent_str: &'a str, quote_keys: bool) -> Self {
        Self { indent_str, quote_keys }
    }

    fn write_indent(&self, writer: &mut String, depth: usize) {
        for _ in 0..depth {
            writer.push_str(self.indent_str);
        }
    }
}

impl<'a> Formatter for PrettyFormatter<'a> {
    fn write_null(&mut self, out: &mut String) -> Result<()> {
        out.push_str("null");
        Ok(())
    }
    fn write_bool(&mut self, out: &mut String, v: bool) -> Result<()> {
        out.push_str(if v { "true" } else { "false" });
        Ok(())
    }
    fn write_number(&mut self, out: &mut String, n: &Number) -> Result<()> {
        out.push_str(&n.to_string());
        Ok(())
    }
    fn write_string(&mut self, out: &mut String, s: &str) -> Result<()> {
        write_escaped_str(out, s, true);
        Ok(())
    }

    fn write_value(&mut self, out: &mut String, v: &Value, depth: usize) -> Result<()> {
        match v {
            Value::Array(arr) => self.write_array(out, arr, depth),
            Value::Object(map) => self.write_object(out, map, depth),
            _ => {
                // Para tipos simples no hay indentación extra aquí
                match v {
                    Value::Null => self.write_null(out),
                    Value::Bool(b) => self.write_bool(out, *b),
                    Value::Number(n) => self.write_number(out, n),
                    Value::String(s) => self.write_string(out, s),
                    _ => unreachable!(),
                }
            },
        }
    }

    fn write_array(&mut self, out: &mut String, arr: &[Value], depth: usize) -> Result<()> {
        if arr.is_empty() {
            out.push_str("[]");
            return Ok(());
        }
        out.push_str("[\n");
        for (i, v) in arr.iter().enumerate() {
            self.write_indent(out, depth + 1);
            self.write_value(out, v, depth + 1)?;
            if i < arr.len() - 1 {
                out.push(',');
            }
            out.push('\n');
        }
        self.write_indent(out, depth);
        out.push(']');
        Ok(())
    }

    fn write_object(&mut self, out: &mut String, obj: &Map<String, Value>, depth: usize) -> Result<()> {
        if obj.is_empty() {
            out.push_str("{}");
            return Ok(());
        }
        out.push_str("{\n");
        for (i, (k, v)) in obj.iter().enumerate() {
            self.write_indent(out, depth + 1);
            self.write_object_key(out, k)?;
            out.push_str(": ");
            self.write_value(out, v, depth + 1)?;
            if i < obj.len() - 1 {
                out.push(',');
            }
            out.push('\n');
        }
        self.write_indent(out, depth);
        out.push('}');
        Ok(())
    }

    fn write_object_key(&mut self, out: &mut String, k: &str) -> Result<()> {
        if !self.quote_keys && is_valid_identifier(k) {
            out.push_str(k);
        } else {
            write_escaped_str(out, k, true);
        }
        Ok(())
    }
}

fn write_escaped_str(out: &mut String, s: &str, quote: bool) {
    if quote {
        out.push('"');
    }
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => {
                let code = c as u32;
                out.push_str("\\u");
                out.push(hex_digit((code >> 12) as u8 & 0xF));
                out.push(hex_digit((code >> 8) as u8 & 0xF));
                out.push(hex_digit((code >> 4) as u8 & 0xF));
                out.push(hex_digit(code as u8 & 0xF));
            },
            c => out.push(c),
        }
    }
    if quote {
        out.push('"');
    }
}

#[inline]
fn is_valid_identifier(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {},
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

#[inline]
fn hex_digit(n: u8) -> char {
    if n < 10 {
        (b'0' + n) as char
    } else {
        (b'a' + n - 10) as char
    }
}

// -------------------------------------------------------------------------
// Value → JSON5 string serializer
// -------------------------------------------------------------------------

pub fn serialize<V>(value: &V) -> Result<String>
where
    V: Serialize,
{
    let value = value.serialize(ValueSerializer)?;
    let mut out = String::with_capacity(256);
    let mut formatter = CompactFormatter::new(false, None);

    formatter.write_value(&mut out, &value, 0)?;
    Ok(out)
}

pub fn serialize_with_formatter<T, V>(value: &V, formatter: &mut T) -> Result<String>
where
    T: Formatter,
    V: Serialize,
{
    let internal_value = value.serialize(ValueSerializer)?;

    let mut out = String::with_capacity(256);
    formatter.write_value(&mut out, &internal_value, 0)?;
    Ok(out)
}
