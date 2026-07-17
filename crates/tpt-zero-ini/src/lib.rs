#![no_std]
//! `tpt-zero-ini`: a small INI parser.
//!
//! Supports the common INI model:
//! - `[section]` headers (no nesting; a later `[section]` simply switches
//!   the current section).
//! - `key = value` or `key : value` assignments (either separator).
//! - `;` and `#` start a comment (to end of line); full-line and trailing
//!   comments are both stripped.
//!
//! The `#![no_std]` core parses line-by-line into a flat sequence of
//! `(section, key, value)` entries written into a caller buffer, with all
//! byte slices borrowed from the input. Behind the `alloc` feature, a
//! [`Document`] model and [`parse`] are provided.
//!
//! # Scope (v0.1)
//!
//! No multi-line values, no escaping, no duplicate-key merging — every
//! assignment becomes its own entry (last-wins is up to the caller).

/// A parsed INI value: raw `key` and `value` bytes (trimmed, comment-stripped)
/// borrowed from the input, plus the `section` they belong to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    /// Current section name, or `None` for entries before the first header.
    pub section: Option<&'a [u8]>,
    /// The key (trimmed bytes, no separator).
    pub key: &'a [u8],
    /// The value (trimmed bytes).
    pub value: &'a [u8],
}

impl<'a> Entry<'a> {
    /// A zeroed placeholder entry (used to fill caller-provided buffers).
    #[inline]
    pub fn placeholder() -> Self {
        Entry {
            section: None,
            key: &[],
            value: &[],
        }
    }
}

/// An error encountered while parsing INI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IniError {
    /// Output buffer (`out`) was too small for the parsed entries.
    TooManyEntries,
    /// A line looked like a `key<sep>value` assignment but had no separator
    /// and was not a section or comment. Offset is the line start.
    InvalidLine(usize),
}

/// Parse `input` into `out`, returning the number of [`Entry`]s written.
///
/// `out` is a caller-provided slice; if there are more entries than
/// `out.len()`, the parse stops with [`IniError::TooManyEntries`]. All byte
/// slices in the returned entries borrow `input`.
#[inline]
pub fn parse_into<'a>(input: &'a [u8], out: &mut [Entry<'a>]) -> Result<usize, IniError> {
    let mut n = 0usize;
    let mut section: Option<&'a [u8]> = None;
    let mut pos = 0usize;
    let len = input.len();

    while pos < len {
        let line_start = pos;
        let mut i = pos;
        while i < len && input[i] != b'\n' {
            i += 1;
        }
        let mut line_end = i;
        if line_end > line_start && input[line_end - 1] == b'\r' {
            line_end -= 1;
        }
        pos = if i < len { i + 1 } else { i };

        let line = &input[line_start..line_end];
        let line = strip_comment(line);
        let trimmed = trim(line);
        if trimmed.is_empty() {
            continue;
        }

        if trimmed[0] == b'[' {
            let close = trimmed.iter().position(|&b| b == b']');
            let name = match close {
                Some(c) if c > 1 => &trimmed[1..c],
                _ => return Err(IniError::InvalidLine(line_start)),
            };
            section = Some(name);
            continue;
        }

        // key<sep>value where sep is '=' or ':'
        let sep = trimmed.iter().position(|&b| b == b'=' || b == b':');
        let (key, value) = match sep {
            Some(s) => {
                let k = trim(&trimmed[..s]);
                let v = trim(&trimmed[s + 1..]);
                (k, v)
            }
            None => return Err(IniError::InvalidLine(line_start)),
        };
        if key.is_empty() {
            return Err(IniError::InvalidLine(line_start));
        }

        if n >= out.len() {
            return Err(IniError::TooManyEntries);
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

/// Strip a `;`/`#` comment (and everything after it) from a line.
#[inline]
fn strip_comment(line: &[u8]) -> &[u8] {
    let mut end = line.len();
    for (i, &b) in line.iter().enumerate() {
        if b == b';' || b == b'#' {
            end = i;
            break;
        }
    }
    &line[..end]
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

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;

    /// A parsed INI document (flat list of `(section, key, value)` entries).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Document {
        pub entries: Vec<(Option<String>, String, String)>,
    }

    impl Document {
        /// Entries in the top-level (pre-first-`[section]`) table.
        pub fn top_level(&self) -> Vec<(&String, &String)> {
            self.entries
                .iter()
                .filter(|(s, _, _)| s.is_none())
                .map(|(_, k, v)| (k, v))
                .collect()
        }

        /// Entries belonging to `section`.
        pub fn section(&self, section: &str) -> Vec<(&String, &String)> {
            self.entries
                .iter()
                .filter(|(s, _, _)| s.as_deref() == Some(section))
                .map(|(_, k, v)| (k, v))
                .collect()
        }
    }

    /// Parse `input` into an owned [`Document`].
    pub fn parse(input: &[u8]) -> Result<Document, IniError> {
        let mut buf = [Entry::placeholder(); 1024];
        let n = parse_into(input, &mut buf)?;
        let mut entries = Vec::with_capacity(n);
        for e in &buf[..n] {
            let section = e
                .section
                .map(|s| String::from_utf8(s.to_vec()).unwrap_or_default());
            let key = String::from_utf8(e.key.to_vec()).unwrap_or_default();
            let value = String::from_utf8(e.value.to_vec()).unwrap_or_default();
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
    fn key_value_separators() {
        let input = b"a = 1\nb: two\nc=three\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 3);
        assert_eq!(out[0].key, b"a");
        assert_eq!(out[0].value, b"1");
        assert_eq!(out[1].key, b"b");
        assert_eq!(out[1].value, b"two");
        assert_eq!(out[2].value, b"three");
    }

    #[test]
    fn sections() {
        let input = b"[sec]\nx = 1\n[other]\ny = 2\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(out[0].section, Some(&b"sec"[..]));
        assert_eq!(out[1].section, Some(&b"other"[..]));
        assert_eq!(out[1].key, b"y");
    }

    #[test]
    fn comments_and_blank() {
        let input = b"; header\n# another\na = 1 ; trailing\n\nb = 2\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(out[0].value, b"1");
        assert_eq!(out[1].value, b"2");
    }

    #[test]
    fn pre_section_entries() {
        let input = b"global = yes\n[sec]\nlocal = no\n";
        let mut out = [Entry::placeholder(); 8];
        let n = parse_into(input, &mut out).unwrap();
        assert_eq!(n, 2);
        assert_eq!(out[0].section, None);
        assert_eq!(out[1].section, Some(&b"sec"[..]));
    }

    #[test]
    fn errors() {
        assert!(matches!(
            parse_into(b"no separator here", &mut [Entry::placeholder(); 8]),
            Err(IniError::InvalidLine(_))
        ));
    }

    #[cfg(feature = "alloc")]
    mod alloc_tests {
        extern crate alloc;
        use super::*;

        #[test]
        fn parse_document() {
            let doc = parse(b"[s]\nx = 1\ny = 2\n").unwrap();
            assert_eq!(doc.entries.len(), 2);
            assert_eq!(doc.section("s").len(), 2);
        }

        #[test]
        fn roundtrip_values() {
            let doc = parse(b"a = hello\nb = world\n").unwrap();
            assert_eq!(doc.top_level().len(), 2);
            assert_eq!(doc.entries[0].1, "a");
            assert_eq!(doc.entries[0].2, "hello");
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
