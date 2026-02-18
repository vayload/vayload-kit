use crate::encoding::json5::ser::{PrettyFormatter, serialize_with_formatter};
use crate::encoding::json5::value::{Number, Value};
use crate::encoding::json5::{from_str, parse_value, to_string, to_string_pretty};
use serde::{Deserialize, Serialize};

#[test]
fn test_null() {
    assert_eq!(parse_value("null").unwrap(), Value::Null);
    assert_eq!(parse_value("  null  ").unwrap(), Value::Null);
}

#[test]
fn test_bool() {
    assert_eq!(parse_value("true").unwrap(), Value::Bool(true));
    assert_eq!(parse_value("false").unwrap(), Value::Bool(false));
    assert_eq!(parse_value("  true  ").unwrap(), Value::Bool(true));
}

#[test]
fn test_integers() {
    assert_eq!(parse_value("0").unwrap(), Value::Number(Number::Int(0)));
    assert_eq!(parse_value("42").unwrap(), Value::Number(Number::Int(42)));
    assert_eq!(parse_value("-17").unwrap(), Value::Number(Number::Int(-17)));
    assert_eq!(
        parse_value("9999999999999999").unwrap(),
        Value::Number(Number::Int(9999999999999999i64))
    );
}

#[test]
fn test_floats() {
    assert_eq!(parse_value("3.14").unwrap(), Value::Number(Number::Float(3.14)));
    assert_eq!(parse_value("-0.5").unwrap(), Value::Number(Number::Float(-0.5)));
    assert_eq!(parse_value("1e10").unwrap(), Value::Number(Number::Float(1e10)));
    assert_eq!(parse_value("1.5E-3").unwrap(), Value::Number(Number::Float(1.5e-3)));
}

#[test]
fn test_json5_special_numbers() {
    assert_eq!(parse_value("NaN").unwrap(), Value::Number(Number::NaN));
    assert_eq!(parse_value("Infinity").unwrap(), Value::Number(Number::Infinity));
    assert_eq!(parse_value("-Infinity").unwrap(), Value::Number(Number::NegInfinity));
    assert_eq!(parse_value("+Infinity").unwrap(), Value::Number(Number::Infinity));
}

#[test]
fn test_hex_numbers() {
    assert_eq!(parse_value("0xFF").unwrap(), Value::Number(Number::Uint(255)));
    assert_eq!(parse_value("0x0").unwrap(), Value::Number(Number::Uint(0)));
    assert_eq!(
        parse_value("0xDEADBEEF").unwrap(),
        Value::Number(Number::Uint(0xDEADBEEF))
    );
    assert_eq!(parse_value("-0x10").unwrap(), Value::Number(Number::Int(-16)));
}

#[test]
fn test_leading_trailing_dot() {
    assert_eq!(parse_value(".5").unwrap(), Value::Number(Number::Float(0.5)));
    assert_eq!(parse_value("5.").unwrap(), Value::Number(Number::Float(5.0)));
}

// -------------------------------------------------------------------------
// String tests
// -------------------------------------------------------------------------

#[test]
fn test_double_quoted_string() {
    assert_eq!(parse_value(r#""hello""#).unwrap(), Value::String("hello".into()));
    assert_eq!(parse_value(r#""""#).unwrap(), Value::String("".into()));
}

#[test]
fn test_single_quoted_string() {
    assert_eq!(parse_value("'hello'").unwrap(), Value::String("hello".into()));
    assert_eq!(parse_value("'it\\'s'").unwrap(), Value::String("it's".into()));
}

#[test]
fn test_escape_sequences() {
    assert_eq!(parse_value(r#""\n\r\t""#).unwrap(), Value::String("\n\r\t".into()));
    assert_eq!(
        parse_value(r#""\b\f\v""#).unwrap(),
        Value::String("\x08\x0C\x0B".into())
    );
    assert_eq!(parse_value(r#""\\""#).unwrap(), Value::String("\\".into()));
    assert_eq!(parse_value(r#""\"""#).unwrap(), Value::String("\"".into()));
    assert_eq!(parse_value(r#""\u0041""#).unwrap(), Value::String("A".into()));
    assert_eq!(parse_value(r#""\u{1F600}""#).unwrap(), Value::String("😀".into()));
    assert_eq!(parse_value(r#""\x41""#).unwrap(), Value::String("A".into()));
}

#[test]
fn test_unicode_surrogate_pair() {
    // 😀 = U+1F600 = surrogate pair D83D DE00
    assert_eq!(parse_value(r#""\uD83D\uDE00""#).unwrap(), Value::String("😀".into()));
}

#[test]
fn test_null_escape() {
    assert_eq!(parse_value(r#""\0""#).unwrap(), Value::String("\0".into()));
}

// -------------------------------------------------------------------------
// Array tests
// -------------------------------------------------------------------------

#[test]
fn test_empty_array() {
    assert_eq!(parse_value("[]").unwrap(), Value::Array(vec![]));
}

#[test]
fn test_simple_array() {
    let v = parse_value("[1, 2, 3]").unwrap();
    assert_eq!(
        v,
        Value::Array(vec![
            Value::Number(Number::Int(1)),
            Value::Number(Number::Int(2)),
            Value::Number(Number::Int(3)),
        ])
    );
}

#[test]
fn test_array_trailing_comma() {
    let v = parse_value("[1, 2, 3,]").unwrap();
    assert_eq!(
        v,
        Value::Array(vec![
            Value::Number(Number::Int(1)),
            Value::Number(Number::Int(2)),
            Value::Number(Number::Int(3)),
        ])
    );
}

#[test]
fn test_nested_array() {
    let v = parse_value("[[1], [2, 3], []]").unwrap();
    assert!(matches!(v, Value::Array(_)));
    if let Value::Array(arr) = v {
        assert_eq!(arr.len(), 3);
    }
}

#[test]
fn test_empty_object() {
    assert_eq!(parse_value("{}").unwrap(), Value::Object(Default::default()));
}

#[test]
fn test_simple_object() {
    let v = parse_value(r#"{"a": 1, "b": 2}"#).unwrap();
    if let Value::Object(m) = v {
        assert_eq!(m.get("a"), Some(&Value::Number(Number::Int(1))));
        assert_eq!(m.get("b"), Some(&Value::Number(Number::Int(2))));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_object_unquoted_keys() {
    let v = parse_value("{foo: 1, bar: 'baz'}").unwrap();
    if let Value::Object(m) = v {
        assert_eq!(m.get("foo"), Some(&Value::Number(Number::Int(1))));
        assert_eq!(m.get("bar"), Some(&Value::String("baz".into())));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_object_single_quoted_keys() {
    let v = parse_value("{'key': 42}").unwrap();
    if let Value::Object(m) = v {
        assert_eq!(m.get("key"), Some(&Value::Number(Number::Int(42))));
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_object_trailing_comma() {
    let v = parse_value("{a: 1, b: 2,}").unwrap();
    assert!(matches!(v, Value::Object(_)));
}

#[test]
fn test_nested_object() {
    let v = parse_value(r#"{"a": {"b": {"c": 42}}}"#).unwrap();
    if let Value::Object(m) = &v {
        if let Some(Value::Object(inner)) = m.get("a") {
            if let Some(Value::Object(deep)) = inner.get("b") {
                assert_eq!(deep.get("c"), Some(&Value::Number(Number::Int(42))));
                return;
            }
        }
    }
    panic!("deep nesting failed");
}

#[test]
fn test_single_line_comment() {
    let v = parse_value("// comment\n42").unwrap();
    assert_eq!(v, Value::Number(Number::Int(42)));
}

#[test]
fn test_multi_line_comment() {
    let v = parse_value("/* comment\n spanning lines */42").unwrap();
    assert_eq!(v, Value::Number(Number::Int(42)));
}

#[test]
fn test_comments_in_object() {
    let v = parse_value("{\n  // comment\n  foo: 1, /* inline */ bar: 2\n}").unwrap();
    if let Value::Object(m) = v {
        assert_eq!(m.len(), 2);
    } else {
        panic!("expected object");
    }
}

#[test]
fn test_comment_before_value() {
    let v = parse_value(
        r#"
    /* JSON5 object */
    {
        // the answer
        answer: 42,
    }
    "#,
    )
    .unwrap();
    if let Value::Object(m) = v {
        assert_eq!(m.get("answer"), Some(&Value::Number(Number::Int(42))));
    }
}

#[test]
fn test_json5_spec_example() {
    let input = r#"{
        // comments
        unquoted: 'and you can quote me on that',
        singleQuotes: 'I can use "double quotes" here',
        lineBreaks: "Look, Mom! \
No \\n's!",
        hexadecimal: 0xdecaf,
        leadingDecimalPoint: .8675309,
        andTrailing: 8675309.,
        positiveSign: +1,
        trailingComma: 'in objects',
        andIn: ['arrays'],
        "backwardsCompatible": "with JSON",
    }"#;

    let v = parse_value(input).unwrap();
    if let Value::Object(m) = &v {
        assert_eq!(
            m.get("unquoted"),
            Some(&Value::String("and you can quote me on that".into()))
        );
        assert_eq!(m.get("hexadecimal"), Some(&Value::Number(Number::Uint(0xdecaf))));
        if let Some(Value::Number(Number::Float(f))) = m.get("leadingDecimalPoint") {
            assert!((f - 0.8675309).abs() < 1e-7);
        } else {
            panic!("leadingDecimalPoint wrong");
        }
        assert_eq!(m.get("backwardsCompatible"), Some(&Value::String("with JSON".into())));
    } else {
        panic!("expected object");
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

#[test]
fn test_deserialize_struct() {
    let p: Point = from_str("{x: 1.0, y: 2.5}").unwrap();
    assert_eq!(p, Point { x: 1.0, y: 2.5 });
}

#[test]
fn test_deserialize_vec() {
    let v: Vec<i32> = from_str("[1, 2, 3, 4]").unwrap();
    assert_eq!(v, vec![1, 2, 3, 4]);
}

#[test]
fn test_deserialize_string() {
    let s: String = from_str("'hello world'").unwrap();
    assert_eq!(s, "hello world");
}

#[test]
fn test_deserialize_option_some() {
    let v: Option<i32> = from_str("42").unwrap();
    assert_eq!(v, Some(42));
}

#[test]
fn test_deserialize_option_none() {
    let v: Option<i32> = from_str("null").unwrap();
    assert_eq!(v, None);
}

#[test]
fn test_deserialize_nested() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Config {
        name: String,
        version: u32,
        tags: Vec<String>,
        debug: bool,
    }

    let s = r#"{
        name: 'my-app',
        version: 2,
        tags: ['rust', 'json5'],
        debug: true,
    }"#;

    let c: Config = from_str(s).unwrap();
    assert_eq!(c.name, "my-app");
    assert_eq!(c.version, 2);
    assert_eq!(c.tags, vec!["rust", "json5"]);
    assert!(c.debug);
}

#[test]
fn test_deserialize_enum_unit() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    let c: Color = from_str(r#""Red""#).unwrap();
    assert_eq!(c, Color::Red);
    let c: Color = from_str("'Blue'").unwrap();
    assert_eq!(c, Color::Blue);
}

#[test]
fn test_deserialize_enum_newtype() {
    #[derive(Debug, Deserialize, PartialEq)]
    enum Wrap {
        Value(i32),
    }

    let w: Wrap = from_str(r#"{"Value": 99}"#).unwrap();
    assert_eq!(w, Wrap::Value(99));
}

#[derive(Serialize)]
struct SPoint {
    x: f64,
    y: f64,
}

#[test]
fn test_serialize_struct() {
    let s = to_string(&SPoint { x: 1.0, y: 2.5 }).unwrap();
    // Round-trip check
    let p: Point = from_str(&s).unwrap();
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 2.5);
}

#[test]
fn test_serialize_vec() {
    let s = to_string(&vec![1i32, 2, 3]).unwrap();
    let v: Vec<i32> = from_str(&s).unwrap();
    assert_eq!(v, vec![1, 2, 3]);
}

#[test]
fn test_serialize_special_floats() {
    let s = to_string(&f64::NAN).unwrap();
    assert_eq!(s, "NaN");
    let s = to_string(&f64::INFINITY).unwrap();
    assert_eq!(s, "Infinity");
    let s = to_string(&f64::NEG_INFINITY).unwrap();
    assert_eq!(s, "-Infinity");
}

#[test]
fn test_pretty_print() {
    let s = to_string_pretty(&SPoint { x: 1.0, y: 2.5 }).unwrap();
    assert!(s.contains('\n'));
    assert!(s.contains("    "), "Expected 4-space indent");
}

#[test]
fn test_roundtrip_complex() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Complex {
        name: String,
        values: Vec<f64>,
        nested: Option<Box<Complex>>,
        flag: bool,
    }

    let original = Complex {
        name: "test".into(),
        values: vec![1.1, 2.2, 3.3],
        nested: Some(Box::new(Complex {
            name: "inner".into(),
            values: vec![],
            nested: None,
            flag: false,
        })),
        flag: true,
    };

    let s = to_string(&original).unwrap();
    let decoded: Complex = from_str(&s).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_error_invalid_json() {
    assert!(parse_value("").is_err());
    assert!(parse_value("{").is_err());
    assert!(parse_value("[1, 2").is_err());
}

#[test]
fn test_error_trailing_data() {
    assert!(parse_value("42 extra").is_err());
}

#[test]
fn test_error_invalid_escape() {
    assert!(parse_value(r#""\q""#).is_err());
}

// -------------------------------------------------------------------------
// Serialize/Deserialize macro tests
// -------------------------------------------------------------------------

#[test]
fn test_serde_simple_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        name: String,
        age: u32,
        active: bool,
    }

    let user = User { name: "Alice".into(), age: 30, active: true };
    let json = to_string(&user).unwrap();
    let decoded: User = from_str(&json).unwrap();
    assert_eq!(user, decoded);
}

#[test]
fn test_serde_nested_structs() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Address {
        city: String,
        zip: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        address: Address,
    }

    let person = Person {
        name: "Bob".into(),
        address: Address { city: "NYC".into(), zip: "10001".into() },
    };
    let json = to_string(&person).unwrap();
    let decoded: Person = from_str(&json).unwrap();
    assert_eq!(person, decoded);
}

#[test]
fn test_serde_enum_variants() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    for status in [Status::Active, Status::Inactive, Status::Pending] {
        let json = to_string(&status).unwrap();
        let decoded: Status = from_str(&json).unwrap();
        assert_eq!(status, decoded);
    }
}

#[test]
fn test_serde_enum_with_data() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(String),
        Color(u8, u8, u8),
    }

    let msgs = vec![
        Message::Quit,
        Message::Move { x: 10, y: 20 },
        Message::Write("hello".into()),
        Message::Color(255, 128, 0),
    ];

    for msg in msgs {
        let json = to_string(&msg).unwrap();
        let decoded: Message = from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }
}

#[test]
fn test_serde_option_types() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Optional {
        required: String,
        optional: Option<i32>,
    }

    let cases = vec![
        Optional { required: "test".into(), optional: Some(42) },
        Optional { required: "test".into(), optional: None },
        Optional { required: "test".into(), optional: Some(0) },
    ];

    for case in cases {
        let json = to_string(&case).unwrap();
        let decoded: Optional = from_str(&json).unwrap();
        assert_eq!(case, decoded);
    }
}

#[test]
fn test_serde_collections() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Collections {
        vec: Vec<i32>,
        empty_vec: Vec<String>,
    }

    let data = Collections { vec: vec![1, 2, 3], empty_vec: vec![] };
    let json = to_string(&data).unwrap();
    let decoded: Collections = from_str(&json).unwrap();
    assert_eq!(data, decoded);
}

#[test]
fn test_serde_tuple() {
    let tuple: (i32, String, bool) = (42, "hello".into(), true);
    let json = to_string(&tuple).unwrap();
    let decoded: (i32, String, bool) = from_str(&json).unwrap();
    assert_eq!(tuple, decoded);
}

#[test]
fn test_serde_unit_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Unit;

    let unit = Unit;
    let json = to_string(&unit).unwrap();
    let decoded: Unit = from_str(&json).unwrap();
    assert_eq!(unit, decoded);
}

#[test]
fn test_serde_newtype_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Wrapper(i32);

    let wrapped = Wrapper(123);
    let json = to_string(&wrapped).unwrap();
    let decoded: Wrapper = from_str(&json).unwrap();
    assert_eq!(wrapped, decoded);
}

#[test]
fn test_serde_escaped_strings() {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Text {
        content: String,
    }

    let cases = vec!["Hello\nWorld", "Tab\there", "Quote: \"test\"", "Backslash: \\", "Unicode: \u{1F600}"];

    for content in cases {
        let text = Text { content: content.into() };
        let json = to_string(&text).unwrap();
        let decoded: Text = from_str(&json).unwrap();
        assert_eq!(text, decoded);
    }
}

#[test]
fn test_indent_default_4_spaces() {
    let obj = SPoint { x: 1.0, y: 2.5 };
    let json = to_string_pretty(&obj).unwrap();
    assert!(json.contains("\n    "), "Expected 4-space indent, got: {}", json);
}

#[test]
fn test_indent_nested_objects() {
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
    }

    #[derive(Serialize)]
    struct Inner {
        value: i32,
    }

    let data = Outer { inner: Inner { value: 42 } };
    let json = to_string_pretty(&data).unwrap();

    println!("{}", json);
    assert!(json.contains("\n    inner"), "Level 1 indent should be 4 spaces");
    assert!(json.contains("\n        value"), "Level 2 indent should be 8 spaces");
}

#[test]
fn test_indent_arrays() {
    let arr = vec![vec![1, 2], vec![3, 4]];
    let json = to_string_pretty(&arr).unwrap();
    assert!(json.contains("\n    ["), "Nested array should have 4-space indent");
    assert!(
        json.contains("\n        1"),
        "Array elements should have 8-space indent"
    );
}

#[test]
fn test_indent_mixed() {
    #[derive(Serialize)]
    struct Mixed {
        items: Vec<Item>,
    }

    #[derive(Serialize)]
    struct Item {
        name: String,
        values: Vec<i32>,
    }

    let data = Mixed {
        items: vec![Item { name: "a".into(), values: vec![1, 2] }, Item { name: "b".into(), values: vec![3, 4] }],
    };

    let json = to_string_pretty(&data).unwrap();
    assert!(
        json.contains("\n    items"),
        "First level should be 4 spaces, got: {}",
        json
    );
    assert!(
        json.contains("\n        {"),
        "Second level array items should be 8 spaces, got: {}",
        json
    );
    assert!(
        json.contains("\n            name"),
        "Third level object keys should be 12 spaces, got: {}",
        json
    );
    assert!(
        json.contains("\n                1"),
        "Fourth level array values should be 16 spaces, got: {}",
        json
    );
}

#[test]
fn parse_with_quoted_keys() {
    #[derive(Deserialize, Serialize)]
    struct QuotedKeys {
        pub name: String,
        pub age: i32,
        pub address: String,
    }

    let json = r#"{
        "name": "John Doe",
        "age": 30,
        "address": "123 Main St"
    }"#;

    let data: QuotedKeys = from_str(json).unwrap();
    assert_eq!(data.name, "John Doe");
    assert_eq!(data.age, 30);
    assert_eq!(data.address, "123 Main St");

    let serialized = serialize_with_formatter(&data, &mut PrettyFormatter::new("    ", true)).unwrap();
    assert!(serialized.contains("\n    \"name\""));
    assert!(serialized.contains("\n    \"age\""));
    assert!(serialized.contains("\n    \"address\""));
}
