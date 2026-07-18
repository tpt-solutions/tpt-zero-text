#![no_std]
//! `out-zero-xml-escape`: XML/HTML entity escaping and unescaping.
//!
//! Handles the five predefined XML entities (`&amp;`, `&lt;`, `&gt;`,
//! `&quot;`, `&apos;`) plus numeric character references (`&#NN;` decimal and
//! `&#xHH;` hex) via `tpt-zero-numstr` (a sibling `no_std` crate).
//!
//! - [`escape_into`] — escape `< > & " '` into a caller buffer.
//! - [`unescape_into`] — decode entities/numeric refs into a caller buffer.
//! - [`escape`] / [`unescape`] (alloc feature) — `String`-returning wrappers.
//!
//! # Scope
//!
//! This is **not** a full HTML5 entity decoder: it supports only the five
//! predefined XML entities and numeric references, not the named-entity table
//! (`&copy;`, `&euro;`, …). That is intentional — it keeps the crate
//! `no_std`, dependency-free, and predictable.

/// The five predefined XML entities.
const ENTITIES: &[(&[u8], u8)] = &[
    (b"amp", b'&'),
    (b"lt", b'<'),
    (b"gt", b'>'),
    (b"quot", b'"'),
    (b"apos", b'\''),
];

/// Look up a predefined entity name (without `&`/`;`), returning the char byte.
#[inline]
fn predefined_entity(name: &[u8]) -> Option<u8> {
    for (n, ch) in ENTITIES {
        if *n == name {
            return Some(*ch);
        }
    }
    None
}

/// Escape `input` into `out`, returning the number of bytes written.
/// Returns `None` if `out` is too small.
#[inline]
pub fn escape_into(input: &[u8], out: &mut [u8]) -> Option<usize> {
    let mut n = 0usize;
    for &b in input {
        let entity: &[u8] = match b {
            b'&' => b"&amp;",
            b'<' => b"&lt;",
            b'>' => b"&gt;",
            b'"' => b"&quot;",
            b'\'' => b"&apos;",
            _ => {
                if n + 1 > out.len() {
                    return None;
                }
                out[n] = b;
                n += 1;
                continue;
            }
        };
        if n + entity.len() > out.len() {
            return None;
        }
        out[n..n + entity.len()].copy_from_slice(entity);
        n += entity.len();
    }
    Some(n)
}

/// Unescape `input` into `out`, returning the number of bytes written.
/// Returns `None` if `out` is too small. Unknown named entities are left
/// verbatim (`&foo;` stays `&foo;`).
#[inline]
pub fn unescape_into(input: &[u8], out: &mut [u8]) -> Option<usize> {
    let mut n = 0usize;
    let len = input.len();
    let mut i = 0usize;
    while i < len {
        let b = input[i];
        if b == b'&' {
            // Try to match a complete entity ending at the next ';'.
            if let Some(semi) = input[i + 1..].iter().position(|&b| b == b';') {
                let inner = &input[i + 1..i + 1 + semi];
                if let Some(ch) = predefined_entity(inner) {
                    if n + 1 > out.len() {
                        return None;
                    }
                    out[n] = ch;
                    n += 1;
                    i += 1 + semi + 1;
                    continue;
                }
                if !inner.is_empty() && inner[0] == b'#' {
                    // numeric char ref
                    let rest = &inner[1..];
                    if let Some(ch) = parse_numeric_ref(rest) {
                        if n + 1 > out.len() {
                            return None;
                        }
                        out[n] = ch;
                        n += 1;
                        i += 1 + semi + 1;
                        continue;
                    }
                }
            }
            // Not a recognized entity: copy the '&' verbatim.
            if n + 1 > out.len() {
                return None;
            }
            out[n] = b'&';
            n += 1;
            i += 1;
        } else {
            if n + 1 > out.len() {
                return None;
            }
            out[n] = b;
            n += 1;
            i += 1;
        }
    }
    Some(n)
}

/// Parse a numeric char ref body: decimal `NN` or hex `xHH`/`XHH`, returning
/// the decoded byte if it fits in a single ASCII byte (this crate only emits
/// bytes, not arbitrary Unicode scalars).
#[inline]
fn parse_numeric_ref(body: &[u8]) -> Option<u8> {
    if body.is_empty() {
        return None;
    }
    let (radix, digits) = if body[0] == b'x' || body[0] == b'X' {
        (16, &body[1..])
    } else {
        (10, body)
    };
    if digits.is_empty() {
        return None;
    }
    let v = tpt_zero_numstr::parse_int::<u32>(digits, radix)?;
    if v <= 0xFF { Some(v as u8) } else { None }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::String;

    /// Escape `input`, returning a `String`.
    pub fn escape(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len() * 5 + 1);
        {
            let mut buf = [0u8; 1024];
            let n = escape_into(input, &mut buf).unwrap_or(0);
            out.push_str(core::str::from_utf8(&buf[..n]).unwrap_or(""));
        }
        out
    }

    /// Unescape `input`, returning a `String`.
    pub fn unescape(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len() + 1);
        {
            let mut buf = [0u8; 1024];
            let n = unescape_into(input, &mut buf).unwrap_or(0);
            out.push_str(core::str::from_utf8(&buf[..n]).unwrap_or(""));
        }
        out
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::{escape, unescape};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_basic() {
        let mut buf = [0u8; 64];
        let n = escape_into(b"<a>&'\"", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"&lt;a&gt;&amp;&apos;&quot;");
    }

    #[test]
    fn unescape_basic() {
        let mut buf = [0u8; 64];
        let n = unescape_into(b"&lt;a&gt;&amp;&apos;&quot;", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"<a>&'\"");
    }

    #[test]
    fn numeric_decimal() {
        let mut buf = [0u8; 64];
        let n = unescape_into(b"&#65;", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"A");
    }

    #[test]
    fn numeric_hex() {
        let mut buf = [0u8; 64];
        let n = unescape_into(b"&#x41;", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"A");
    }

    #[test]
    fn unknown_entity_verbatim() {
        let mut buf = [0u8; 64];
        let n = unescape_into(b"&copy;", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"&copy;");
    }

    #[test]
    fn escape_roundtrip() {
        let input = b"<tag attr=\"v&x\">text</tag>";
        let mut e = [0u8; 128];
        let n = escape_into(input, &mut e).unwrap();
        let mut d = [0u8; 128];
        let m = unescape_into(&e[..n], &mut d).unwrap();
        assert_eq!(&d[..m], input);
    }

    #[test]
    fn buffer_too_small() {
        let mut buf = [0u8; 2];
        assert_eq!(escape_into(b"<", &mut buf), None);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn alloc_wrappers() {
        assert_eq!(escape(b"<>&"), "&lt;&gt;&amp;");
        assert_eq!(unescape(b"&lt;&gt;&amp;"), "<>&");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Escaping then unescaping always round-trips for the supported set.
        #[test]
        fn roundtrip(input in proptest::collection::vec(any::<u8>(), 0..256)) {
            let mut e = [0u8; 1024];
            let n = escape_into(&input, &mut e).unwrap();
            let mut d = [0u8; 1024];
            let m = unescape_into(&e[..n], &mut d).unwrap();
            prop_assert_eq!(&d[..m], &input[..]);
        }

        /// Unescaping well-formed predefined entities is idempotent-ish: every
        /// `&` in the output is part of an unrecognized entity.
        #[test]
        fn unescape_no_predefined_left(s in "[<>&'\"]*") {
            let bytes = s.as_bytes();
            let mut d = [0u8; 1024];
            let m = unescape_into(bytes, &mut d).unwrap();
            let out = &d[..m];
            prop_assert!(!out.windows(5).any(|w| w == b"&amp;"));
            prop_assert!(!out.windows(4).any(|w| w == b"&lt;"));
            prop_assert!(!out.windows(4).any(|w| w == b"&gt;"));
        }
    }
}
