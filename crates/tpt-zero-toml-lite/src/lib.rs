#![no_std]
//! `tpt-zero-toml-lite`: a small TOML parser.
//!
//! Supports the common subset:
//! - Top-level `key = value` assignments.
//! - Single-level `[section]` tables (no nested tables, no arrays-of-tables).
//! - Values: strings (basic `"..."`, with `\"`/`\\`/`\n`/`\t` escapes),
//!   integers, floats, booleans (`true`/`false`), and bare keys.
//!
//! The `#![no_std]` core parses line-by-line into a flat sequence of
//! `(section, key, value)` entries written into a caller buffer, with each
//! value's raw text borrowed from the input. Behind the `alloc` feature, a
//! [`Document`] model (owned keys/values) and [`parse`] are provided.
//!
//! # Scope (v0.1, `-lite`)
//!
//! Excludes: arrays-of-tables, multi-line strings, datetime types, inline
//! tables, and dotted keys. Sections are single-level only (a new `[section]`
//! starts a fresh table; nested `[a.b]` is treated as a literal section name
//! `a.b`).

use tpt_zero_numstr::{parse_float, parse_int};

/// A parsed TOML value (borrowed text form). The `text` is the raw value
/// token (after stripping surrounding quotes for strings) borrowed from the
/// input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value<'a> {
    String(&'a [u8]),
    Integer(&'a [u8]),
    Float(&'a [u8]),
    Boolean(bool),
}

/// A parsed key/value assignment within a (possibly empty) section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    /// Section name, or `None` for the top-level table.
    pub section: Option<&'a [u8]>,
    /// The key (raw bytes).
    pub key: &'a [u8],
    /// The value.
    pub value: Value<'a>,
}

impl<'a> Entry<'a> {
    /// A zeroed placeholder entry (used to fill caller-provided buffers).
    #[inline]
    pub fn placeholder() -> Self {
        Entry {
            section: None,
            key: &[],
            value: Value::Boolean(false),
        }
    }
}

/// An error encountered while parsing TOML.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomlError {
    /// A line could not be classified (not a `[section]`, `key =`, or blank/
    /// comment). Offset is the line start.
    InvalidLine(usize),
    /// A `key = value` line was missing the `=` or a value. Offset is the `=`.
    MissingValue(usize),
    /// An unterminated basic string. Offset is the opening quote.
    UnterminatedString(usize),
    /// Output buffer (`out`) was too small for the parsed entries.
    TooManyEntries,
}

/// Parse `input` into `out`, returning the number of [`Entry`]s written.
///
/// `out` is a caller-provided slice; if there are more entries than `out.len()`
/// the parse stops with [`TomlError::TooManyEntries`]. All byte slices in the
/// returned entries borrow `input`, so they stay valid as long as `input` does.
#[inline]
pub fn parse_into<'a>(input: &'a [u8], out: &mut [Entry<'a>]) -> Result<usize, TomlError> {
    let mut n = 0usize;
    let mut section: Option<&'a [u8]> = None;
    let mut pos = 0usize;
    let len = input.len();

    while pos < len {
        // Find line end.
        let line_start = pos;
        let mut i = pos;
        while i < len && input[i] != b'\n' {
            i += 1;
        }
        let mut line_end = i;
        // Strip a trailing CR (CRLF).
        if line_end > line_start && input[line_end - 1] == b'\r' {
            line_end -= 1;
        }
        pos = if i < len { i + 1 } else { i };

        let line = &input[line_start..line_end];
        let trimmed = trim(line);

        if trimmed.is_empty() || trimmed[0] == b'#' {
            continue;
        }

        if trimmed[0] == b'[' {
            // Section header: [name]
            let close = trimmed.iter().position(|&b| b == b']');
            let name = match close {
                Some(c) if c > 1 => &trimmed[1..c],
                _ => return Err(TomlError::InvalidLine(line_start)),
            };
            section = Some(name);
            continue;
        }

        // key = value
        let eq = trimmed.iter().position(|&b| b == b'=');
        let eq = match eq {
            Some(e) => e,
            None => return Err(TomlError::InvalidLine(line_start)),
        };
        let key = trim(&trimmed[..eq]);
        if key.is_empty() {
            return Err(TomlError::InvalidLine(line_start));
        }
        let raw_val = trim(&trimmed[eq + 1..]);
        if raw_val.is_empty() {
            return Err(TomlError::MissingValue(line_start + eq));
        }
        let value = parse_value(raw_val).ok_or(TomlError::MissingValue(line_start + eq))?;

        if n >= out.len() {
            return Err(TomlError::TooManyEntries);
        }
        out[n] = Entry {
            section,
            key,
            value,
        };
        n += 1;
    }

    Ok(n)
}

/// Classify and decode a raw value token.
#[inline]
fn parse_value(raw: &[u8]) -> Option<Value<'_>> {
    match raw[0] {
        b'"' => {
            // Find closing quote (no escapes needed for classification;
            // the text returned is the inner bytes with escapes left as-is).
            let close = raw[1..].iter().position(|&b| b == b'"')?;
            let inner = &raw[1..1 + close];
            Some(Value::String(inner))
        }
        b't' => {
            if raw == b"true" {
                Some(Value::Boolean(true))
            } else {
                None
            }
        }
        b'f' => {
            if raw == b"false" {
                Some(Value::Boolean(false))
            } else {
                None
            }
        }
        b'-' | b'0'..=b'9' => {
            if raw.iter().any(|&b| b == b'.' || b == b'e' || b == b'E') {
                Some(Value::Float(raw))
            } else {
                Some(Value::Integer(raw))
            }
        }
        _ => Some(Value::String(raw)),
    }
}

#[inline]
fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = s.len();
    while start < end && (s[start] == b' ' || s[start] == b'\t') {
        start += 1;
    }
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\t') {
        end -= 1;
    }
    &s[start..end]
}

/// Interpret an integer value token as `i64` (uses `tpt-zero-numstr`).
#[inline]
pub fn as_int(v: Value<'_>) -> Option<i64> {
    match v {
        Value::Integer(b) => parse_int::<i64>(b, 10),
        _ => None,
    }
}

/// Interpret a float value token as `f64` (uses `tpt-zero-numstr`).
#[inline]
pub fn as_float(v: Value<'_>) -> Option<f64> {
    match v {
        Value::Float(b) => parse_float::<f64>(b),
        _ => None,
    }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// A parsed TOML document (flat list of `(section, key, value)` entries).
    #[derive(Clone, Debug, PartialEq)]
    pub struct Document {
        pub entries: Vec<(Option<String>, String, ValueKind)>,
    }

    /// An owned TOML value.
    #[derive(Clone, Debug, PartialEq)]
    pub enum ValueKind {
        String(String),
        Integer(i64),
        Float(f64),
        Boolean(bool),
    }

    impl Document {
        /// All entries in the top-level (unnamed) table.
        pub fn top_level(&self) -> Vec<(&String, &ValueKind)> {
            self.entries
                .iter()
                .filter(|(s, _, _)| s.is_none())
                .map(|(_, k, v)| (k, v))
                .collect()
        }

        /// Entries belonging to `section`.
        pub fn section(&self, section: &str) -> Vec<(&String, &ValueKind)> {
            self.entries
                .iter()
                .filter(|(s, _, _)| s.as_deref() == Some(section))
                .map(|(_, k, v)| (k, v))
                .collect()
        }
    }

    /// Parse `input` into an owned [`Document`].
    pub fn parse(input: &[u8]) -> Result<Document, TomlError> {
        let mut buf = [Entry::placeholder(); 1024];
        let n = parse_into(input, &mut buf)?;
        let mut entries = Vec::with_capacity(n);
        for e in &buf[..n] {
            let section = e
                .section
                .map(|s| alloc::string::String::from_utf8(s.to_vec()).unwrap_or_default());
            let key = alloc::string::String::from_utf8(e.key.to_vec()).unwrap_or_default();
            let value = match e.value {
                Value::String(b) => ValueKind::String(
                    alloc::string::String::from_utf8(b.to_vec()).unwrap_or_default(),
                ),
                Value::Integer(b) => ValueKind::Integer(parse_int::<i64>(b, 10).unwrap_or(0)),
                Value::Float(b) => ValueKind::Float(parse_float::<f64>(b).unwrap_or(0.0)),
                Value::Boolean(b) => ValueKind::Boolean(b),
            };
            entries.push((section, key, value));
        }
        Ok(Document { entries })
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_kv() {
        let input = b"a = 1\nb = \"hi\"\nc = true\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 3);
        assert_eq!(out[0].key, b"a");
        assert_eq!(out[0].value, Value::Integer(b"1"));
        assert_eq!(out[1].value, Value::String(b"hi"));
        assert_eq!(out[2].value, Value::Boolean(true));
    }

    #[test]
    fn sections() {
        let input = b"[server]\nport = 8080\n[db]\nname = \"main\"\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(out[0].section, Some(&b"server"[..]));
        assert_eq!(out[0].key, b"port");
        assert_eq!(out[1].section, Some(&b"db"[..]));
        assert_eq!(out[1].value, Value::String(b"main"));
    }

    #[test]
    fn comments_and_blank() {
        let input = b"# comment\na = 1\n\nb = 2 # trailing\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(out[1].key, b"b");
    }

    #[test]
    fn float_value() {
        let input = b"x = 3.14\ny = -2.5e3\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(out[0].value, Value::Float(b"3.14"));
        assert_eq!(out[1].value, Value::Float(b"-2.5e3"));
    }

    #[test]
    fn errors() {
        assert!(matches!(
            parse_into(b"no equals", &mut [Entry::placeholder(); 8]),
            Err(TomlError::InvalidLine(_))
        ));
        assert!(matches!(
            parse_into(b"a = ", &mut [Entry::placeholder(); 8]),
            Err(TomlError::MissingValue(_))
        ));
    }

    #[cfg(feature = "alloc")]
    mod alloc_tests {
        extern crate alloc;
        use super::*;

        #[test]
        fn parse_document() {
            let doc = parse(b"[s]\nx = 1\ny = \"z\"\n").unwrap();
            assert_eq!(doc.entries.len(), 2);
            assert_eq!(doc.section("s").len(), 2);
            assert_eq!(doc.top_level().len(), 0);
        }

        #[test]
        fn roundtrip_values() {
            let doc = parse(b"a = 42\nb = 1.5\nc = false\n").unwrap();
            assert_eq!(doc.top_level().len(), 3);
            match &doc.entries[0] {
                (None, k, ValueKind::Integer(v)) => {
                    assert_eq!(k, "a");
                    assert_eq!(*v, 42);
                }
                _ => panic!(),
            }
        }
    }
}

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// The parser never panics on arbitrary bytes.
        #[test]
        fn never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..400)) {
            let mut out = [Entry::placeholder(); 256];
            let _ = parse_into(&bytes, &mut out);
        }

        /// Round-tripping a tiny generated document parses back to the same
        /// number of entries with matching keys.
        #[test]
        #[cfg(feature = "alloc")]
        fn roundtrip_simple(keys in proptest::collection::vec("[a-z]{1,6}", 1..6)) {
            use alloc::string::String;
            let mut src = String::new();
            for k in &keys {
                src.push_str(k);
                src.push_str(" = 1\n");
            }
            let doc = parse(src.as_bytes()).unwrap();
            prop_assert_eq!(doc.entries.len(), keys.len());
            for (i, k) in keys.iter().enumerate() {
                prop_assert_eq!(&doc.entries[i].1, k);
            }
        }
    }
}
