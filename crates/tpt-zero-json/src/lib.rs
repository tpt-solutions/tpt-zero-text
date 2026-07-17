#![no_std]
//! `tpt-zero-json`: a small JSON tokenizer and value model.
//!
//! The `#![no_std]` core provides a streaming, allocation-free
//! [`Tokenizer`]. It yields one [`Token`] at a time and borrows its input
//! bytes, so it never allocates and only keeps a small amount of scratch
//! state. Strings are emitted lazily: if a string has no escape sequences
//! ([`Token::StringRaw`]) the slice is returned by reference (a zero-copy fast
//! path); if it contains escapes, repeated calls to [`Tokenizer::next_token`]
//! emit [`Token::StringChunk`]s carrying decoded bytes via an internal buffer,
//! ending with `last == true`.
//!
//! Behind the `alloc` feature, a [`Value`] model is provided plus
//! [`from_slice`] / [`to_string`] for full parse/serialize. Object entries use
//! `Vec<(String, Value)>` to **preserve insertion order** (not a hash map).
//!
//! # Limitations (v0.1)
//!
//! The `alloc` layer does not enforce a maximum nesting depth, size limits, or
//! number canonicalization — it is intended for small trusted-ish documents.
//! Duplicate object keys keep all entries (last-wins is up to the caller).

use tpt_zero_numstr::{parse_float, parse_int};

/// A lexical token emitted by the [`Tokenizer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// `{`
    ObjectStart,
    /// `}`
    ObjectEnd,
    /// `[`
    ArrayStart,
    /// `]`
    ArrayEnd,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// A literal `true`.
    True,
    /// A literal `false`.
    False,
    /// A literal `null`.
    Null,
    /// A number (raw bytes; use [`tpt_zero_numstr`] to parse). `negative`
    /// reflects a leading `-`.
    Number { bytes: &'a [u8], negative: bool },
    /// A string with no escape sequences: the raw (valid UTF-8) slice.
    StringRaw(&'a [u8]),
    /// A decoded chunk of an escaped string. `last` is `true` for the final
    /// chunk of the current string. Bytes are already unescaped. The slice
    /// lives until the next call to [`Tokenizer::next_token`].
    StringChunk { bytes: &'a [u8], last: bool },
    /// End of input. Always the final token.
    Eof,
}

/// An error encountered while tokenizing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonError {
    /// Invalid UTF-8 byte sequence in a string body.
    InvalidUtf8(usize),
    /// A control character (0x00..=0x1F) appeared unescaped in a string.
    ControlCharInString(usize),
    /// A malformed escape sequence (`\` not followed by a valid escape, or a
    /// bad `\u` hex digit).
    BadEscape(usize),
    /// A malformed number (empty or trailing junk).
    BadNumber(usize),
    /// Unexpected byte (not a valid JSON token start).
    UnexpectedByte(usize),
    /// Input ended in the middle of a token/value.
    UnexpectedEof,
}

/// Scratch buffer size for decoded escape chunks.
const CHUNK_BUF: usize = 64;

/// A streaming JSON tokenizer. Borrows its input; never allocates memory.
#[derive(Clone, Copy)]
pub struct Tokenizer<'a> {
    src: &'a [u8],
    pos: usize,
    /// Active string-decode state: `(body_start, scan)` where `scan` is the
    /// next unread byte of the string body.
    decode: Option<(usize, usize)>,
    /// Reusable scratch buffer for decoded escape chunks. The lifetime is
    /// erased by re-borrowing through `next_token`'s output.
    scratch: [u8; CHUNK_BUF],
}

impl<'a> Tokenizer<'a> {
    /// Create a tokenizer over `src`.
    #[inline]
    pub fn new(src: &'a [u8]) -> Self {
        Tokenizer {
            src,
            pos: 0,
            decode: None,
            scratch: [0u8; CHUNK_BUF],
        }
    }

    /// Current byte offset into the source.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Return the next token. Call repeatedly until [`Token::Eof`].
    ///
    /// When a string contains escape sequences, the tokenizer emits one
    /// [`Token::StringChunk`] per call until the string ends (`last == true`).
    /// Each chunk borrows an internal scratch buffer that is overwritten on
    /// the next call, so copy it out if you need to retain it.
    #[inline]
    pub fn next_token(&mut self) -> Result<Token<'a>, JsonError> {
        // Mid-string decode: emit the next chunk (or final chunk).
        if let Some((body_start, scan)) = self.decode {
            return self.next_string_chunk(body_start, scan);
        }
        self.skip_ws();
        if self.pos >= self.src.len() {
            return Ok(Token::Eof);
        }
        let b = self.src[self.pos];
        match b {
            b'{' => self.tok(Token::ObjectStart, 1),
            b'}' => self.tok(Token::ObjectEnd, 1),
            b'[' => self.tok(Token::ArrayStart, 1),
            b']' => self.tok(Token::ArrayEnd, 1),
            b',' => self.tok(Token::Comma, 1),
            b':' => self.tok(Token::Colon, 1),
            b't' => self.lit(b"true", Token::True),
            b'f' => self.lit(b"false", Token::False),
            b'n' => self.lit(b"null", Token::Null),
            b'"' => self.string_start(),
            b'-' | b'0'..=b'9' => self.number(),
            _ => Err(JsonError::UnexpectedByte(self.pos)),
        }
    }

    #[inline]
    fn tok(&mut self, t: Token<'a>, adv: usize) -> Result<Token<'a>, JsonError> {
        self.pos += adv;
        Ok(t)
    }

    #[inline]
    fn lit(&mut self, word: &[u8], t: Token<'a>) -> Result<Token<'a>, JsonError> {
        if self.src[self.pos..].len() < word.len()
            || &self.src[self.pos..self.pos + word.len()] != word
        {
            return Err(JsonError::UnexpectedByte(self.pos));
        }
        self.tok(t, word.len())
    }

    #[inline]
    fn skip_ws(&mut self) {
        while self.pos < self.src.len() {
            match self.src[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                _ => break,
            }
        }
    }

    #[inline]
    fn number(&mut self) -> Result<Token<'a>, JsonError> {
        let start = self.pos;
        let negative = self.src[self.pos] == b'-';
        if negative {
            self.pos += 1;
        }
        let int_start = self.pos;
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos == int_start {
            return Err(JsonError::BadNumber(start));
        }
        if self.pos < self.src.len() && self.src[self.pos] == b'.' {
            self.pos += 1;
            let frac_start = self.pos;
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == frac_start {
                return Err(JsonError::BadNumber(start));
            }
        }
        if self.pos < self.src.len() && (self.src[self.pos] == b'e' || self.src[self.pos] == b'E') {
            self.pos += 1;
            if self.pos < self.src.len()
                && (self.src[self.pos] == b'+' || self.src[self.pos] == b'-')
            {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while self.pos < self.src.len() && self.src[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            if self.pos == exp_start {
                return Err(JsonError::BadNumber(start));
            }
        }
        Ok(Token::Number {
            bytes: &self.src[start..self.pos],
            negative,
        })
    }

    #[inline]
    fn string_start(&mut self) -> Result<Token<'a>, JsonError> {
        let start = self.pos;
        self.pos += 1; // consume opening quote
        let body_start = self.pos;
        let mut i = self.pos;
        while i < self.src.len() {
            let b = self.src[i];
            if b == b'"' {
                let raw = &self.src[body_start..i];
                if !is_valid_utf8_substring(raw) {
                    return Err(JsonError::InvalidUtf8(start));
                }
                self.pos = i + 1;
                return Ok(Token::StringRaw(raw));
            }
            if b == b'\\' {
                // Escape present: enter chunked decode starting at body_start.
                self.decode = Some((body_start, body_start));
                return self.next_string_chunk(body_start, body_start);
            }
            if b < 0x20 {
                return Err(JsonError::ControlCharInString(i));
            }
            i = utf8_next(self.src, i);
        }
        Err(JsonError::UnexpectedEof)
    }

    #[inline]
    fn next_string_chunk(
        &mut self,
        body_start: usize,
        scan: usize,
    ) -> Result<Token<'a>, JsonError> {
        let mut n = 0usize;
        let mut s = scan;
        while s < self.src.len() && n < CHUNK_BUF {
            let b = self.src[s];
            if b == b'"' {
                // End of string. If buffer empty and we are at the close quote
                // immediately, emit a trailing empty chunk marked last.
                if n == 0 {
                    self.pos = s + 1;
                    self.decode = None;
                    return Ok(Token::StringChunk {
                        bytes: &[],
                        last: true,
                    });
                }
                self.pos = s + 1;
                self.decode = None;
                return Ok(Token::StringChunk {
                    bytes: unsafe { core::slice::from_raw_parts(self.scratch.as_ptr(), n) },
                    last: true,
                });
            }
            if b == b'\\' {
                s += 1;
                if s >= self.src.len() {
                    return Err(JsonError::BadEscape(s));
                }
                let e = self.src[s];
                match e {
                    b'"' => self.scratch[n] = b'"',
                    b'\\' => self.scratch[n] = b'\\',
                    b'/' => self.scratch[n] = b'/',
                    b'b' => self.scratch[n] = 0x08,
                    b'f' => self.scratch[n] = 0x0C,
                    b'n' => self.scratch[n] = b'\n',
                    b'r' => self.scratch[n] = b'\r',
                    b't' => self.scratch[n] = b'\t',
                    b'u' => {
                        let cp = self.decode_unicode_escape(s)?;
                        s += 5; // past `\uXXXX`
                        let written = encode_utf8(cp, &mut self.scratch[n..]);
                        n += written;
                        continue;
                    }
                    _ => return Err(JsonError::BadEscape(s)),
                }
                n += 1;
                s += 1;
            } else if b < 0x20 {
                return Err(JsonError::ControlCharInString(s));
            } else {
                self.scratch[n] = b;
                n += 1;
                s = utf8_next(self.src, s);
            }
        }
        if s >= self.src.len() {
            return Err(JsonError::UnexpectedEof);
        }
        // Buffer full: emit a non-final chunk and resume next call.
        self.decode = Some((body_start, s));
        Ok(Token::StringChunk {
            bytes: unsafe { core::slice::from_raw_parts(self.scratch.as_ptr(), n) },
            last: false,
        })
    }

    #[inline]
    fn decode_unicode_escape(&self, backslash_pos: usize) -> Result<u32, JsonError> {
        let hex_start = backslash_pos + 1;
        if hex_start + 4 > self.src.len() {
            return Err(JsonError::BadEscape(backslash_pos));
        }
        let mut cp = 0u32;
        for k in 0..4 {
            let d = hex_val(self.src[hex_start + k]).ok_or(JsonError::BadEscape(backslash_pos))?;
            cp = (cp << 4) | d;
        }
        // Minimal surrogate support: lone surrogates are passed through as-is.
        Ok(cp)
    }
}

/// Advance `i` past the UTF-8 code point starting at `i` (returns the index of
/// the next code-point boundary). Falls back to `i + 1` past the end.
#[inline]
fn utf8_next(src: &[u8], i: usize) -> usize {
    if i >= src.len() {
        return src.len();
    }
    let b = src[i];
    let step = if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1
    };
    (i + step).min(src.len())
}

#[inline]
fn hex_val(b: u8) -> Option<u32> {
    match b {
        b'0'..=b'9' => Some((b - b'0') as u32),
        b'a'..=b'f' => Some((b - b'a' + 10) as u32),
        b'A'..=b'F' => Some((b - b'A' + 10) as u32),
        _ => None,
    }
}

/// Encode a Unicode scalar as UTF-8 into `out`, returning the byte count.
#[inline]
fn encode_utf8(cp: u32, out: &mut [u8]) -> usize {
    if cp < 0x80 {
        if out.is_empty() {
            return 0;
        }
        out[0] = cp as u8;
        1
    } else if cp < 0x800 {
        if out.len() < 2 {
            return 0;
        }
        out[0] = 0xC0 | ((cp >> 6) as u8);
        out[1] = 0x80 | ((cp & 0x3F) as u8);
        2
    } else if cp < 0x1_0000 {
        if out.len() < 3 {
            return 0;
        }
        out[0] = 0xE0 | ((cp >> 12) as u8);
        out[1] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        out[2] = 0x80 | ((cp & 0x3F) as u8);
        3
    } else {
        if out.len() < 4 {
            return 0;
        }
        out[0] = 0xF0 | ((cp >> 18) as u8);
        out[1] = 0x80 | (((cp >> 12) & 0x3F) as u8);
        out[2] = 0x80 | (((cp >> 6) & 0x3F) as u8);
        out[3] = 0x80 | ((cp & 0x3F) as u8);
        4
    }
}

/// Validate that `s` is a well-formed UTF-8 byte sequence (substring of a
/// larger buffer). We only need to confirm no incomplete trailing sequence.
#[inline]
fn is_valid_utf8_substring(s: &[u8]) -> bool {
    tpt_zero_utf8::from_bytes(s).is_ok()
}

/// Parse a JSON number token's bytes into an `f64`. Integers use
/// [`parse_int`] (exact); everything else falls back to [`parse_float`].
#[inline]
pub fn parse_number(bytes: &[u8]) -> Option<f64> {
    let is_int = !bytes.iter().any(|&b| b == b'.' || b == b'e' || b == b'E');
    if is_int {
        parse_int::<i64>(bytes, 10).map(|v| v as f64)
    } else {
        parse_float::<f64>(bytes)
    }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    /// A JSON value. Object entries preserve insertion order via a `Vec`.
    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    impl Value {
        /// Index into an array or object (object lookup by key). Returns `None`
        /// for non-indexable variants or missing keys.
        pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
            index.index_into(self)
        }

        /// Convenience: treat as a `&str`.
        pub fn as_str(&self) -> Option<&str> {
            match self {
                Value::String(s) => Some(s),
                _ => None,
            }
        }

        /// Convenience: treat as an `f64`.
        pub fn as_f64(&self) -> Option<f64> {
            match self {
                Value::Number(n) => Some(*n),
                _ => None,
            }
        }

        /// Convenience: treat as a `bool`.
        pub fn as_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(b) => Some(*b),
                _ => None,
            }
        }

        /// Convenience: treat as an array slice.
        pub fn as_array(&self) -> Option<&[Value]> {
            match self {
                Value::Array(a) => Some(a),
                _ => None,
            }
        }

        /// Convenience: treat as object entries.
        pub fn as_object(&self) -> Option<&[(String, Value)]> {
            match self {
                Value::Object(o) => Some(o),
                _ => None,
            }
        }
    }

    /// Indexing helper trait for [`Value::get`].
    pub trait Index {
        fn index_into<'b>(&self, value: &'b Value) -> Option<&'b Value>;
    }

    impl Index for usize {
        fn index_into<'b>(&self, value: &'b Value) -> Option<&'b Value> {
            match value {
                Value::Array(a) => a.get(*self),
                _ => None,
            }
        }
    }

    impl Index for str {
        fn index_into<'b>(&self, value: &'b Value) -> Option<&'b Value> {
            match value {
                Value::Object(o) => o.iter().find(|(k, _)| k == self).map(|(_, v)| v),
                _ => None,
            }
        }
    }

    impl Index for &str {
        fn index_into<'b>(&self, value: &'b Value) -> Option<&'b Value> {
            (*self).index_into(value)
        }
    }

    /// A thin token stream wrapper that supports one token of lookahead, so
    /// parsers can peek without a `put_back` on the underlying [`Tokenizer`].
    struct Reader<'a> {
        tok: Tokenizer<'a>,
        peeked: Option<Token<'a>>,
    }

    impl<'a> Reader<'a> {
        fn new(tok: Tokenizer<'a>) -> Self {
            Reader { tok, peeked: None }
        }
        fn next(&mut self) -> Result<Token<'a>, JsonError> {
            if let Some(t) = self.peeked.take() {
                Ok(t)
            } else {
                self.tok.next_token()
            }
        }
        fn peek(&mut self) -> Result<Token<'a>, JsonError> {
            if self.peeked.is_none() {
                self.peeked = Some(self.tok.next_token()?);
            }
            Ok(self.peeked.unwrap())
        }
    }

    /// Parse `input` into a [`Value`].
    pub fn from_slice(input: &[u8]) -> Result<Value, JsonError> {
        let mut r = Reader::new(Tokenizer::new(input));
        let v = parse_value(&mut r)?;
        match r.next()? {
            Token::Eof => Ok(v),
            _ => Err(JsonError::UnexpectedByte(r.tok.position())),
        }
    }

    fn parse_value(r: &mut Reader<'_>) -> Result<Value, JsonError> {
        match r.next()? {
            Token::ObjectStart => parse_object(r),
            Token::ArrayStart => parse_array(r),
            Token::True => Ok(Value::Bool(true)),
            Token::False => Ok(Value::Bool(false)),
            Token::Null => Ok(Value::Null),
            Token::Number { bytes, .. } => {
                let n = parse_number(bytes).ok_or(JsonError::BadNumber(r.tok.position()))?;
                Ok(Value::Number(n))
            }
            Token::StringRaw(s) => Ok(Value::String(
                core::str::from_utf8(s).unwrap_or("").to_string(),
            )),
            Token::StringChunk { bytes, last } => {
                let mut s = String::new();
                if let Ok(text) = core::str::from_utf8(bytes) {
                    s.push_str(text);
                }
                if last {
                    Ok(Value::String(s))
                } else {
                    Ok(Value::String(collect_string_rest(s, r)?))
                }
            }
            Token::Eof => Err(JsonError::UnexpectedEof),
            _ => Err(JsonError::UnexpectedByte(r.tok.position())),
        }
    }

    fn collect_string_rest(mut s: String, r: &mut Reader<'_>) -> Result<String, JsonError> {
        loop {
            match r.next()? {
                Token::StringChunk { bytes, last } => {
                    if let Ok(text) = core::str::from_utf8(bytes) {
                        s.push_str(text);
                    }
                    if last {
                        return Ok(s);
                    }
                }
                _ => return Err(JsonError::UnexpectedByte(r.tok.position())),
            }
        }
    }

    fn parse_array(r: &mut Reader<'_>) -> Result<Value, JsonError> {
        let mut items = Vec::new();
        if r.peek()? == Token::ArrayEnd {
            r.next()?;
            return Ok(Value::Array(items));
        }
        loop {
            items.push(parse_value(r)?);
            match r.next()? {
                Token::Comma => continue,
                Token::ArrayEnd => return Ok(Value::Array(items)),
                _ => return Err(JsonError::UnexpectedByte(r.tok.position())),
            }
        }
    }

    fn parse_object(r: &mut Reader<'_>) -> Result<Value, JsonError> {
        let mut entries: Vec<(String, Value)> = Vec::new();
        if r.peek()? == Token::ObjectEnd {
            r.next()?;
            return Ok(Value::Object(entries));
        }
        loop {
            let key = match r.next()? {
                Token::StringRaw(s) => core::str::from_utf8(s).unwrap_or("").to_string(),
                Token::StringChunk { bytes, last } => {
                    let mut ks = String::new();
                    if let Ok(text) = core::str::from_utf8(bytes) {
                        ks.push_str(text);
                    }
                    if last {
                        ks
                    } else {
                        collect_string_rest(ks, r)?
                    }
                }
                _ => return Err(JsonError::UnexpectedByte(r.tok.position())),
            };
            match r.next()? {
                Token::Colon => {}
                _ => return Err(JsonError::UnexpectedByte(r.tok.position())),
            }
            let val = parse_value(r)?;
            entries.push((key, val));
            match r.next()? {
                Token::Comma => continue,
                Token::ObjectEnd => return Ok(Value::Object(entries)),
                _ => return Err(JsonError::UnexpectedByte(r.tok.position())),
            }
        }
    }

    /// Serialize `value` to a JSON `String`.
    pub fn to_string(value: &Value) -> String {
        let mut out = String::new();
        write_value(value, &mut out);
        out
    }

    fn write_value(v: &Value, out: &mut String) {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(true) => out.push_str("true"),
            Value::Bool(false) => out.push_str("false"),
            Value::Number(n) => {
                if let Some(i) = to_int_if_whole(*n) {
                    let mut buf = [0u8; 24];
                    if let Some(s) = tpt_zero_numstr::format_int(i, 10, &mut buf) {
                        out.push_str(core::str::from_utf8(s).unwrap());
                        return;
                    }
                }
                let mut buf = [0u8; 32];
                if let Some(s) = tpt_zero_numstr::format_float(*n, &mut buf) {
                    out.push_str(core::str::from_utf8(s).unwrap());
                } else {
                    out.push_str("null");
                }
            }
            Value::String(s) => write_string(s, out),
            Value::Array(a) => {
                out.push('[');
                for (i, item) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_value(item, out);
                }
                out.push(']');
            }
            Value::Object(o) => {
                out.push('{');
                for (i, (k, val)) in o.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_string(k, out);
                    out.push(':');
                    write_value(val, out);
                }
                out.push('}');
            }
        }
    }

    fn to_int_if_whole(n: f64) -> Option<i64> {
        if n.is_finite() && n < 9.0e15 && n > -9.0e15 {
            let i = n as i64;
            // Confirm the conversion was exact (no rounding / fractional part).
            if (i as f64) == n {
                return Some(i);
            }
        }
        None
    }

    fn write_string(s: &str, out: &mut String) {
        out.push('"');
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\u{08}' => out.push_str("\\b"),
                '\u{0C}' => out.push_str("\\f"),
                c if (c as u32) < 0x20 => {
                    out.push_str("\\u");
                    out.push_str(&format_hex4(c as u32));
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }

    fn format_hex4(v: u32) -> String {
        let mut s = String::with_capacity(4);
        for shift in (0..4).rev() {
            let d = ((v >> (shift * 4)) & 0xF) as u8;
            s.push(if d < 10 {
                (b'0' + d) as char
            } else {
                (b'a' + d - 10) as char
            });
        }
        s
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizer_raw_string_fast_path() {
        let mut t = Tokenizer::new(br#"  "hello"  "#);
        assert_eq!(t.next_token().unwrap(), Token::StringRaw(b"hello"));
        assert_eq!(t.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn tokenizer_literals_and_punct() {
        let mut t = Tokenizer::new(br#"{ "a": true, "b": false, "c": null }"#);
        assert_eq!(t.next_token().unwrap(), Token::ObjectStart);
        assert_eq!(t.next_token().unwrap(), Token::StringRaw(b"a"));
        assert_eq!(t.next_token().unwrap(), Token::Colon);
        assert_eq!(t.next_token().unwrap(), Token::True);
        assert_eq!(t.next_token().unwrap(), Token::Comma);
        assert_eq!(t.next_token().unwrap(), Token::StringRaw(b"b"));
        assert_eq!(t.next_token().unwrap(), Token::Colon);
        assert_eq!(t.next_token().unwrap(), Token::False);
        assert_eq!(t.next_token().unwrap(), Token::Comma);
        assert_eq!(t.next_token().unwrap(), Token::StringRaw(b"c"));
        assert_eq!(t.next_token().unwrap(), Token::Colon);
        assert_eq!(t.next_token().unwrap(), Token::Null);
        assert_eq!(t.next_token().unwrap(), Token::ObjectEnd);
        assert_eq!(t.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn tokenizer_numbers() {
        let mut t = Tokenizer::new(b"123 -4.5 1.2e3 0");
        assert_eq!(
            t.next_token().unwrap(),
            Token::Number {
                bytes: b"123",
                negative: false
            }
        );
        assert_eq!(
            t.next_token().unwrap(),
            Token::Number {
                bytes: b"-4.5",
                negative: true
            }
        );
        assert_eq!(
            t.next_token().unwrap(),
            Token::Number {
                bytes: b"1.2e3",
                negative: false
            }
        );
        assert_eq!(
            t.next_token().unwrap(),
            Token::Number {
                bytes: b"0",
                negative: false
            }
        );
    }

    #[test]
    fn tokenizer_escaped_string() {
        let mut t = Tokenizer::new(br#""a\tb\n\"c""#);
        match t.next_token().unwrap() {
            Token::StringChunk { bytes, last } => {
                assert_eq!(bytes, b"a\tb\n\"c");
                assert!(last);
            }
            other => panic!("unexpected token: {other:?}"),
        }
    }

    #[test]
    fn tokenizer_unicode_escape() {
        let mut t = Tokenizer::new(br#""\u0041""#);
        match t.next_token().unwrap() {
            Token::StringChunk { bytes, last } => {
                assert_eq!(bytes, b"A");
                assert!(last);
            }
            other => panic!("unexpected token: {other:?}"),
        }
    }

    #[test]
    fn parse_number_helper() {
        assert_eq!(parse_number(b"42"), Some(42.0));
        assert_eq!(parse_number(b"-3.5"), Some(-3.5));
        assert_eq!(parse_number(b"1.5e2"), Some(150.0));
    }

    #[test]
    fn errors() {
        assert!(matches!(
            Tokenizer::new(b"\"\x01\"").next_token(),
            Err(JsonError::ControlCharInString(_))
        ));
        assert!(matches!(
            Tokenizer::new(b"\"\\x\"").next_token(),
            Err(JsonError::BadEscape(_))
        ));
    }

    #[cfg(feature = "alloc")]
    mod alloc_tests {
        extern crate alloc;
        use super::*;

        #[test]
        fn parse_simple() {
            let v = from_slice(br#"{"a":1,"b":[true,false,null],"c":"hi"}"#).unwrap();
            assert_eq!(v.get("a").and_then(|x| x.as_f64()), Some(1.0));
            assert_eq!(v.get("b").and_then(|x| x.as_array()).unwrap().len(), 3);
            assert_eq!(v.get("c").and_then(|x| x.as_str()), Some("hi"));
        }

        #[test]
        fn roundtrip() {
            let src = br#"{"name":"tpt","n":42,"ok":true,"list":[1,2,3]}"#;
            let v = from_slice(src).unwrap();
            let s = to_string(&v);
            let v2 = from_slice(s.as_bytes()).unwrap();
            assert_eq!(v, v2);
        }

        #[test]
        fn escape_roundtrip() {
            let v = Value::String(alloc::string::String::from("a\tb\nc"));
            let s = to_string(&v);
            if let Err(e) = from_slice(s.as_bytes()) {
                panic!("parse failed: {:?} on serialized {s:?}", e);
            }
            let v2 = from_slice(s.as_bytes()).unwrap();
            assert_eq!(v, v2);
        }

        #[test]
        fn parse_escapes_explicit() {
            let v = from_slice(br#""a\tb\nc""#).unwrap();
            assert_eq!(v, Value::String(alloc::string::String::from("a\tb\nc")));
        }

        #[test]
        fn whitespace_then_eof() {
            assert_eq!(from_slice(b"  123  ").unwrap(), Value::Number(123.0));
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Tokenizer never panics on arbitrary bytes.
        #[test]
        fn never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
            let mut t = Tokenizer::new(&bytes);
            for _ in 0..512 {
                match t.next_token() {
                    Ok(Token::Eof) => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }

        /// A plain ASCII-ish string always tokenizes to a `StringRaw` equal to
        /// the input (no escapes, valid UTF-8 we generate).
        #[test]
        fn raw_string_roundtrip(s in "[!#-\\[\\]-~]{0,64}") {
            let mut json = [0u8; 130];
            let mut n = 0usize;
            json[n] = b'"';
            n += 1;
            for &b in s.as_bytes() {
                json[n] = b;
                n += 1;
            }
            json[n] = b'"';
            n += 1;
            let mut t = Tokenizer::new(&json[..n]);
            match t.next_token() {
                Ok(Token::StringRaw(raw)) => {
                    prop_assert_eq!(core::str::from_utf8(raw).unwrap(), &s);
                }
                other => assert!(matches!(other, Ok(Token::StringChunk { .. }))),
            }
        }

        /// `parse_number` is consistent: formatting the parsed `f64` then
        /// re-parsing yields the same `f64` for integer-valued inputs.
        #[test]
        fn number_parse_stable(i in -1_000_000i64..1_000_000) {
            let mut buf = [0u8; 24];
            let s = tpt_zero_numstr::format_int(i, 10, &mut buf).unwrap();
            let v = parse_number(s).unwrap();
            prop_assert_eq!(v, i as f64);
        }
    }
}
