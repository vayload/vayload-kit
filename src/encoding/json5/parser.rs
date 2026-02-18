/// High-performance JSON5 parser operating on raw bytes.
/// Works on &[u8] to avoid UTF-8 validation overhead in the hot path.
use crate::encoding::json5::error::{Error, Result};
use crate::encoding::json5::value::{Map, Number, Value};

pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    #[inline]
    pub fn new(input: &'a str) -> Self {
        Self { input: input.as_bytes(), pos: 0 }
    }

    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.input.len() - self.pos
    }

    #[inline(always)]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    #[inline(always)]
    fn peek2(&self) -> Option<u8> {
        self.input.get(self.pos + 1).copied()
    }

    #[inline(always)]
    fn advance(&mut self) {
        self.pos += 1;
    }

    #[inline(always)]
    fn eat(&mut self) -> Option<u8> {
        let b = self.input.get(self.pos).copied();
        self.pos += 1;
        b
    }

    #[inline(always)]
    fn expect(&mut self, b: u8) -> Result<()> {
        match self.peek() {
            Some(c) if c == b => {
                self.advance();
                Ok(())
            },
            Some(c) => Err(Error::Expected(b as char, Some(c as char))),
            None => Err(Error::UnexpectedEof),
        }
    }

    pub fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip standard whitespace + JSON5 Unicode whitespace/line terminators
            while let Some(b) = self.peek() {
                match b {
                    b' ' | b'\t' | b'\n' | b'\r' => self.advance(),
                    0xC2 => {
                        // Could be U+00A0 (NBSP): 0xC2 0xA0
                        if self.input.get(self.pos + 1).copied() == Some(0xA0) {
                            self.pos += 2;
                        } else {
                            break;
                        }
                    },
                    0xE2 => {
                        // Could be various Unicode spaces (U+2028, U+2029, etc.)
                        // 0xE2 0x80 {0x80..=0xAF, 0xA8, 0xA9}
                        #[allow(clippy::collapsible_if)]
                        if let (Some(b1), Some(b2)) = (
                            self.input.get(self.pos + 1).copied(),
                            self.input.get(self.pos + 2).copied(),
                        ) {
                            if b1 == 0x80 && (b2 == 0xA8 || b2 == 0xA9 || (0x80..=0xAF).contains(&b2)) {
                                self.pos += 3;
                                continue;
                            }
                        }
                        break;
                    },
                    _ => break,
                }
            }

            // Check for comments
            match (self.peek(), self.peek2()) {
                (Some(b'/'), Some(b'/')) => {
                    // Single-line comment: skip until newline
                    self.pos += 2;
                    while let Some(b) = self.peek() {
                        if b == b'\n' || b == b'\r' {
                            break;
                        }
                        // Handle Unicode line terminators (U+2028, U+2029)
                        #[allow(clippy::collapsible_if)]
                        if b == 0xE2 {
                            if let (Some(0x80), Some(b2)) = (
                                self.input.get(self.pos + 1).copied(),
                                self.input.get(self.pos + 2).copied(),
                            ) {
                                if b2 == 0xA8 || b2 == 0xA9 {
                                    break;
                                }
                            }
                        }
                        self.advance();
                    }
                },
                (Some(b'/'), Some(b'*')) => {
                    // Multi-line comment: skip until */
                    self.pos += 2;
                    loop {
                        match (self.peek(), self.peek2()) {
                            (Some(b'*'), Some(b'/')) => {
                                self.pos += 2;
                                break;
                            },
                            (None, _) => break, // unclosed comment - lenient
                            _ => self.advance(),
                        }
                    }
                },
                _ => break,
            }
        }
    }

    pub fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace_and_comments();
        match self.peek().ok_or(Error::UnexpectedEof)? {
            b'n' => self.parse_null(),
            b't' | b'f' => self.parse_bool(),
            b'"' | b'\'' => self.parse_string_value(),
            b'[' => self.parse_array(),
            b'{' => self.parse_object(),
            b'-' => {
                // Could be negative number or -Infinity
                if self.input.get(self.pos + 1..self.pos + 9) == Some(b"Infinity") {
                    self.pos += 9;
                    Ok(Value::Number(Number::NegInfinity))
                } else {
                    self.parse_number()
                }
            },
            b'+' => {
                // JSON5 allows +Infinity
                if self.input.get(self.pos + 1..self.pos + 9) == Some(b"Infinity") {
                    self.pos += 9;
                    Ok(Value::Number(Number::Infinity))
                } else {
                    self.parse_number()
                }
            },
            b'I' => {
                // Infinity
                if self.input.get(self.pos..self.pos + 8) == Some(b"Infinity") {
                    self.pos += 8;
                    Ok(Value::Number(Number::Infinity))
                } else {
                    Err(Error::UnexpectedChar('I', self.pos))
                }
            },
            b'N' => {
                // NaN
                if self.input.get(self.pos..self.pos + 3) == Some(b"NaN") {
                    self.pos += 3;
                    Ok(Value::Number(Number::NaN))
                } else {
                    Err(Error::UnexpectedChar('N', self.pos))
                }
            },
            b'0'..=b'9' | b'.' => self.parse_number(),
            c => Err(Error::UnexpectedChar(c as char, self.pos)),
        }
    }

    // -------------------------------------------------------------------------
    // Null
    // -------------------------------------------------------------------------

    fn parse_null(&mut self) -> Result<Value> {
        if self.input.get(self.pos..self.pos + 4) == Some(b"null") {
            self.pos += 4;
            Ok(Value::Null)
        } else {
            Err(Error::UnexpectedChar('n', self.pos))
        }
    }

    // -------------------------------------------------------------------------
    // Boolean
    // -------------------------------------------------------------------------

    fn parse_bool(&mut self) -> Result<Value> {
        if self.input.get(self.pos..self.pos + 4) == Some(b"true") {
            self.pos += 4;
            Ok(Value::Bool(true))
        } else if self.input.get(self.pos..self.pos + 5) == Some(b"false") {
            self.pos += 5;
            Ok(Value::Bool(false))
        } else {
            Err(Error::UnexpectedChar(self.peek().unwrap_or(0) as char, self.pos))
        }
    }

    // -------------------------------------------------------------------------
    // String  (JSON5 adds single-quoted strings, multi-line via backslash, more escapes)
    // -------------------------------------------------------------------------

    fn parse_string_value(&mut self) -> Result<Value> {
        Ok(Value::String(self.parse_string()?))
    }

    pub fn parse_string(&mut self) -> Result<String> {
        let quote = self.eat().ok_or(Error::UnexpectedEof)?;
        debug_assert!(quote == b'"' || quote == b'\'');
        self.parse_string_contents(quote)
    }

    fn parse_string_contents(&mut self, quote: u8) -> Result<String> {
        // Fast path: scan ahead for end quote without escapes
        let start = self.pos;
        let mut has_escape = false;

        loop {
            match self.peek() {
                None => return Err(Error::UnexpectedEof),
                Some(b) if b == quote => {
                    let end = self.pos;
                    self.advance();
                    if !has_escape {
                        // Zero-copy fast path
                        return Ok(std::str::from_utf8(&self.input[start..end])
                            .map_err(|_| Error::Custom("Invalid UTF-8 in string".into()))?
                            .to_owned());
                    }
                    break; // fall through to slow path rebuild
                },
                Some(b'\\') => {
                    has_escape = true;
                    self.advance();
                    self.advance();
                },
                Some(b'\n') | Some(b'\r') if quote != b'\'' => {
                    return Err(Error::UnexpectedChar('\n', self.pos));
                },
                Some(b) if b < 0x20 => {
                    return Err(Error::UnexpectedChar(b as char, self.pos));
                },
                _ => self.advance(),
            }
        }

        // Slow path: rebuild with escapes resolved
        self.pos = start;
        let mut out = String::with_capacity(64);
        loop {
            match self.peek() {
                None => return Err(Error::UnexpectedEof),
                Some(b) if b == quote => {
                    self.advance();
                    return Ok(out);
                },
                Some(b'\\') => {
                    self.advance();
                    self.parse_escape(&mut out)?;
                },
                Some(b) => {
                    // Decode UTF-8 char
                    let ch = self.decode_utf8_char()?;
                    // JSON5 allows line continuation in strings
                    if ch == '\n' || ch == '\r' {
                        // line terminator in string is an error unless escaped
                        return Err(Error::UnexpectedChar(ch, self.pos));
                    }
                    // JSON5: U+2028 / U+2029 are allowed in strings
                    out.push(ch);
                    let _ = b;
                },
            }
        }
    }

    fn decode_utf8_char(&mut self) -> Result<char> {
        let b0 = self.eat().ok_or(Error::UnexpectedEof)?;
        let ch = if b0 < 0x80 {
            b0 as char
        } else if b0 & 0xE0 == 0xC0 {
            let b1 = self.eat().ok_or(Error::UnexpectedEof)?;
            let cp = ((b0 & 0x1F) as u32) << 6 | (b1 & 0x3F) as u32;
            char::from_u32(cp).ok_or(Error::InvalidUnicode(cp))?
        } else if b0 & 0xF0 == 0xE0 {
            let b1 = self.eat().ok_or(Error::UnexpectedEof)?;
            let b2 = self.eat().ok_or(Error::UnexpectedEof)?;
            let cp = ((b0 & 0x0F) as u32) << 12 | ((b1 & 0x3F) as u32) << 6 | (b2 & 0x3F) as u32;
            char::from_u32(cp).ok_or(Error::InvalidUnicode(cp))?
        } else {
            let b1 = self.eat().ok_or(Error::UnexpectedEof)?;
            let b2 = self.eat().ok_or(Error::UnexpectedEof)?;
            let b3 = self.eat().ok_or(Error::UnexpectedEof)?;
            let cp = ((b0 & 0x07) as u32) << 18
                | ((b1 & 0x3F) as u32) << 12
                | ((b2 & 0x3F) as u32) << 6
                | (b3 & 0x3F) as u32;
            char::from_u32(cp).ok_or(Error::InvalidUnicode(cp))?
        };
        Ok(ch)
    }

    fn parse_escape(&mut self, out: &mut String) -> Result<()> {
        let b = self.eat().ok_or(Error::UnexpectedEof)?;
        match b {
            b'"' => out.push('"'),
            b'\'' => out.push('\''),
            b'\\' => out.push('\\'),
            b'/' => out.push('/'),
            b'b' => out.push('\x08'),
            b'f' => out.push('\x0C'),
            b'n' => out.push('\n'),
            b'r' => out.push('\r'),
            b't' => out.push('\t'),
            b'v' => out.push('\x0B'), // JSON5: vertical tab
            b'0' => {
                // Null escape, but only if not followed by digit
                if matches!(self.peek(), Some(b'1'..=b'9')) {
                    return Err(Error::InvalidEscape('0'));
                }
                out.push('\0');
            },
            b'u' => {
                let cp = self.parse_unicode_escape()?;
                out.push(cp);
            },
            b'x' => {
                // JSON5: hex escape \xNN
                let hi = self.eat_hex_digit()?;
                let lo = self.eat_hex_digit()?;
                let cp = (hi << 4) | lo;
                out.push(char::from_u32(cp as u32).ok_or(Error::InvalidUnicode(cp as u32))?);
            },
            b'\n' | b'\r' => {
                // JSON5: line continuation — skip line terminator
                if b == b'\r' && self.peek() == Some(b'\n') {
                    self.advance();
                }
                // continuation just means the newline is ignored
            },
            // Invalid escape sequence - reject unknown escapes
            _ => return Err(Error::InvalidEscape(b as char)),
        }
        Ok(())
    }

    fn parse_unicode_escape(&mut self) -> Result<char> {
        // Support both \uXXXX and \u{XXXXX} (ES6 style)
        if self.peek() == Some(b'{') {
            self.advance();
            let mut cp: u32 = 0;
            let mut digits = 0;
            loop {
                match self.peek() {
                    Some(b'}') => {
                        self.advance();
                        break;
                    },
                    Some(b) => {
                        let d = hex_val(b).ok_or(Error::InvalidEscape('u'))?;
                        cp = (cp << 4) | d as u32;
                        digits += 1;
                        if digits > 6 {
                            return Err(Error::InvalidUnicode(cp));
                        }
                        self.advance();
                    },
                    None => return Err(Error::UnexpectedEof),
                }
            }
            char::from_u32(cp).ok_or(Error::InvalidUnicode(cp))
        } else {
            let mut cp: u32 = 0;
            for _ in 0..4 {
                let b = self.eat().ok_or(Error::UnexpectedEof)?;
                let d = hex_val(b).ok_or(Error::InvalidEscape('u'))?;
                cp = (cp << 4) | d as u32;
            }
            // Handle surrogate pairs
            if (0xD800..=0xDBFF).contains(&cp) {
                // High surrogate — expect \uXXXX low surrogate
                if self.peek() == Some(b'\\') && self.peek2() == Some(b'u') {
                    self.pos += 2;
                    let mut lo: u32 = 0;
                    for _ in 0..4 {
                        let b = self.eat().ok_or(Error::UnexpectedEof)?;
                        let d = hex_val(b).ok_or(Error::InvalidEscape('u'))?;
                        lo = (lo << 4) | d as u32;
                    }
                    if !(0xDC00..=0xDFFF).contains(&lo) {
                        return Err(Error::InvalidUnicode(lo));
                    }
                    let full = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                    return char::from_u32(full).ok_or(Error::InvalidUnicode(full));
                }
            }
            char::from_u32(cp).ok_or(Error::InvalidUnicode(cp))
        }
    }

    fn eat_hex_digit(&mut self) -> Result<u8> {
        let b = self.eat().ok_or(Error::UnexpectedEof)?;
        hex_val(b).ok_or(Error::InvalidEscape('x'))
    }

    // -------------------------------------------------------------------------
    // Number
    // -------------------------------------------------------------------------

    fn parse_number(&mut self) -> Result<Value> {
        let start = self.pos;
        let negative = self.peek() == Some(b'-');
        if negative || self.peek() == Some(b'+') {
            self.advance();
        }

        // Hexadecimal: 0x / 0X
        if self.peek() == Some(b'0') && matches!(self.peek2(), Some(b'x') | Some(b'X')) {
            self.pos += 2;
            let hex_start = self.pos;
            while matches!(
                self.peek(),
                Some(b'0'..=b'9') | Some(b'a'..=b'f') | Some(b'A'..=b'F') | Some(b'_')
            ) {
                self.advance();
            }
            let hex_str: String =
                self.input[hex_start..self.pos].iter().filter(|&&b| b != b'_').map(|&b| b as char).collect();
            let n = u64::from_str_radix(&hex_str, 16).map_err(|_| Error::InvalidNumber(hex_str.clone()))?;
            if negative {
                return Ok(Value::Number(Number::Int(-(n as i64))));
            }
            return Ok(Value::Number(Number::Uint(n)));
        }

        let mut is_float = false;
        let mut has_exp = false;

        // Integer part
        if self.peek() == Some(b'0') {
            self.advance();
        } else {
            while matches!(self.peek(), Some(b'0'..=b'9') | Some(b'_')) {
                self.advance();
            }
        }

        // Fractional part
        if self.peek() == Some(b'.') {
            is_float = true;
            self.advance();
            // JSON5 allows leading/trailing dot: .5 and 5.
            while matches!(self.peek(), Some(b'0'..=b'9') | Some(b'_')) {
                self.advance();
            }
        }

        // Exponent
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            is_float = true;
            has_exp = true;
            self.advance();
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.advance();
            }
            while matches!(self.peek(), Some(b'0'..=b'9') | Some(b'_')) {
                self.advance();
            }
        }
        let _ = has_exp;

        // Build clean number string (strip underscores — JSON5 doesn't allow them
        // but we handle them gracefully; actual JSON5 only allows them in identifiers)
        let raw = &self.input[start..self.pos];
        let s: String = raw.iter().filter(|&&b| b != b'_').map(|&b| b as char).collect();

        if is_float {
            let f: f64 = s.parse().map_err(|_| Error::InvalidNumber(s.clone()))?;
            Ok(Value::Number(Number::Float(f)))
        } else if negative {
            let i: i64 = s.parse().map_err(|_| Error::InvalidNumber(s.clone()))?;
            Ok(Value::Number(Number::Int(i)))
        } else {
            // Use Int for small positive numbers, Uint for large ones
            match s.parse::<u64>() {
                Ok(n) if n <= i64::MAX as u64 => Ok(Value::Number(Number::Int(n as i64))),
                Ok(n) => Ok(Value::Number(Number::Uint(n))),
                Err(_) => {
                    let f: f64 = s.parse().map_err(|_| Error::InvalidNumber(s.clone()))?;
                    Ok(Value::Number(Number::Float(f)))
                },
            }
        }
    }

    // -------------------------------------------------------------------------
    // Array
    // -------------------------------------------------------------------------

    fn parse_array(&mut self) -> Result<Value> {
        self.expect(b'[')?;
        let mut arr = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            match self.peek() {
                None => return Err(Error::UnexpectedEof),
                Some(b']') => {
                    self.advance();
                    return Ok(Value::Array(arr));
                },
                _ => {},
            }

            arr.push(self.parse_value()?);
            self.skip_whitespace_and_comments();

            match self.peek() {
                Some(b',') => {
                    self.advance();
                    // JSON5: trailing commas allowed
                },
                Some(b']') => {},
                Some(c) => return Err(Error::UnexpectedChar(c as char, self.pos)),
                None => return Err(Error::UnexpectedEof),
            }
        }
    }

    fn parse_object(&mut self) -> Result<Value> {
        self.expect(b'{')?;
        let mut map = Map::new();

        loop {
            self.skip_whitespace_and_comments();
            match self.peek() {
                None => return Err(Error::UnexpectedEof),
                Some(b'}') => {
                    self.advance();
                    return Ok(Value::Object(map));
                },
                _ => {},
            }

            let key = self.parse_key()?;
            self.skip_whitespace_and_comments();
            self.expect(b':')?;
            let value = self.parse_value()?;
            map.insert(key, value);

            self.skip_whitespace_and_comments();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                    // trailing commas allowed in JSON5
                },
                Some(b'}') => {},
                Some(c) => return Err(Error::UnexpectedChar(c as char, self.pos)),
                None => return Err(Error::UnexpectedEof),
            }
        }
    }

    /// JSON5 keys can be quoted strings OR unquoted identifiers
    /// Supports to normal JSON
    fn parse_key(&mut self) -> Result<String> {
        match self.peek() {
            Some(b'"') | Some(b'\'') => self.parse_string(),
            Some(b) if is_id_start(b) => self.parse_identifier(),
            // Handle Unicode identifier starts (e.g. accented chars)
            Some(b) if b >= 0x80 => self.parse_identifier(),
            Some(c) => Err(Error::UnexpectedChar(c as char, self.pos)),
            None => Err(Error::UnexpectedEof),
        }
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let mut s = String::with_capacity(16);
        // First char
        let ch = self.decode_utf8_char()?;
        if !is_id_start_char(ch) {
            return Err(Error::UnexpectedChar(ch, self.pos));
        }
        s.push(ch);

        loop {
            match self.peek() {
                None => break,
                Some(b) if is_id_continue(b) => {
                    s.push(b as char);
                    self.advance();
                },
                Some(b) if b >= 0x80 => {
                    let ch = self.decode_utf8_char()?;
                    if is_id_continue_char(ch) {
                        s.push(ch);
                    } else {
                        // Put back
                        self.pos -= ch.len_utf8();
                        break;
                    }
                },
                _ => break,
            }
        }
        Ok(s)
    }
}

#[inline(always)]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[inline(always)]
fn is_id_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_' || b == b'$'
}

#[inline(always)]
fn is_id_start_char(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == '$'
}

#[inline(always)]
fn is_id_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[inline(always)]
fn is_id_continue_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$' || c == '\u{200C}' || c == '\u{200D}'
}
