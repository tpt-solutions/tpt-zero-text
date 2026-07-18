#![no_std]
//! `out-zero-url-encode`: percent-encoding and decoding.
//!
//! Implements RFC 3986 percent-encoding against the **unreserved** set
//! (`A-Z a-z 0-9 - _ . ~`), which never need encoding, plus opt-in encode
//! sets for characters commonly needing encoding in query strings and
//! component paths. The core API is buffer-writer based so it works in
//! `#![no_std]` without heap allocation.
//!
//! - [`percent_encode_into`] — encode `input` into a caller-provided buffer.
//! - [`percent_decode_into`] — decode `%XX` escapes into a caller buffer.
//! - [`encode`] / [`decode`] (alloc feature) — `String`-returning wrappers.
//!
//! # Decoding caveat
//!
//! Decoding validates that each `%` is followed by two hex digits. A bare `%`
//! at end-of-input is treated as a literal `%` (lenient) so that arbitrary
//! already-encoded text round-trips via [`percent_decode_into`] →
//! [`percent_encode_into`].

/// Whether `b` is in the RFC 3986 unreserved set (never needs encoding).
#[inline]
fn is_unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~')
}

/// Whether `b` needs encoding under the `Component` set (encodes everything
/// except unreserved plus `/ : @`).
#[inline]
fn is_component_ok(b: u8) -> bool {
    is_unreserved(b) || matches!(b, b'/' | b':' | b'@')
}

/// Whether `b` needs encoding under the `Query` set (keeps a few extra chars).
#[inline]
fn is_query_ok(b: u8) -> bool {
    is_unreserved(b)
        || matches!(
            b,
            b'/' | b':' | b'@' | b'&' | b'=' | b'+' | b',' | b';' | b'*' | b'\''
        )
}

/// Encode-set selector controlling aggressiveness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeSet {
    /// Encode everything except the RFC 3986 unreserved set.
    Full,
    /// Encode everything except the unreserved set plus `/ : @` (path-like).
    Component,
    /// Encode everything except the unreserved set plus common query chars.
    Query,
}

impl EncodeSet {
    #[inline]
    fn keep(&self, b: u8) -> bool {
        match self {
            EncodeSet::Full => is_unreserved(b),
            EncodeSet::Component => is_component_ok(b),
            EncodeSet::Query => is_query_ok(b),
        }
    }
}

/// Encode `input` bytes into `out`, returning the number of bytes written.
/// Returns `None` if `out` is too small.
#[inline]
pub fn percent_encode_into(input: &[u8], set: EncodeSet, out: &mut [u8]) -> Option<usize> {
    let mut n = 0usize;
    for &b in input {
        if set.keep(b) {
            if n + 1 > out.len() {
                return None;
            }
            out[n] = b;
            n += 1;
        } else {
            if n + 3 > out.len() {
                return None;
            }
            out[n] = b'%';
            out[n + 1] = hex_digit(b >> 4);
            out[n + 2] = hex_digit(b & 0x0F);
            n += 3;
        }
    }
    Some(n)
}

/// Decode percent-encoded `input` into `out`, returning the number of bytes
/// written. Returns `None` if `out` is too small. A malformed `%` escape is
/// copied through verbatim (lenient).
#[inline]
pub fn percent_decode_into(input: &[u8], out: &mut [u8]) -> Option<usize> {
    let mut n = 0usize;
    let mut i = 0usize;
    let len = input.len();
    while i < len {
        let b = input[i];
        if b == b'%' && i + 2 < len {
            let hi = hex_val(input[i + 1]);
            let lo = hex_val(input[i + 2]);
            if let (Some(h), Some(l)) = (hi, lo) {
                if n + 1 > out.len() {
                    return None;
                }
                out[n] = (h << 4) | l;
                n += 1;
                i += 3;
                continue;
            }
        }
        if n + 1 > out.len() {
            return None;
        }
        out[n] = b;
        n += 1;
        i += 1;
    }
    Some(n)
}

#[inline]
fn hex_digit(v: u8) -> u8 {
    if v < 10 { b'0' + v } else { b'A' + (v - 10) }
}

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::String;

    /// Percent-encode `input`, returning a `String`.
    pub fn encode(input: &[u8], set: EncodeSet) -> String {
        // Worst-case expansion is 3x; allocate to fit.
        let mut out = String::with_capacity(input.len() * 3 + 1);
        {
            let mut buf = [0u8; 768];
            let n = percent_encode_into(input, set, &mut buf).unwrap_or(0);
            out.push_str(core::str::from_utf8(&buf[..n]).unwrap_or(""));
        }
        out
    }

    /// Percent-decode `input`, returning a `String`.
    pub fn decode(input: &[u8]) -> String {
        let mut out = String::with_capacity(input.len() + 1);
        {
            let mut buf = [0u8; 1024];
            let n = percent_decode_into(input, &mut buf).unwrap_or(0);
            out.push_str(core::str::from_utf8(&buf[..n]).unwrap_or(""));
        }
        out
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::{decode, encode};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_unreserved_passthrough() {
        let mut buf = [0u8; 64];
        let n = percent_encode_into(b"abc-_.~XYZ", EncodeSet::Full, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"abc-_.~XYZ");
    }

    #[test]
    fn encode_spaces_and_symbols() {
        let mut buf = [0u8; 64];
        let n = percent_encode_into(b"a b&c=d", EncodeSet::Full, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"a%20b%26c%3Dd");
    }

    #[test]
    fn component_keeps_slash() {
        let mut buf = [0u8; 64];
        let n = percent_encode_into(b"a/b c", EncodeSet::Component, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"a/b%20c");
    }

    #[test]
    fn decode_basic() {
        let mut buf = [0u8; 64];
        let n = percent_decode_into(b"a%20b%26c", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"a b&c");
    }

    #[test]
    fn decode_lowercase_hex() {
        let mut buf = [0u8; 64];
        let n = percent_decode_into(b"%3d", &mut buf).unwrap();
        assert_eq!(&buf[..n], b"=");
    }

    #[test]
    fn roundtrip_full() {
        let input = b"hello world/foo?bar=baz#frag";
        let mut enc = [0u8; 128];
        let n = percent_encode_into(input, EncodeSet::Full, &mut enc).unwrap();
        let mut dec = [0u8; 128];
        let m = percent_decode_into(&enc[..n], &mut dec).unwrap();
        assert_eq!(&dec[..m], input);
    }

    #[test]
    fn buffer_too_small() {
        let mut buf = [0u8; 2];
        assert_eq!(percent_encode_into(b"abc", EncodeSet::Full, &mut buf), None);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn alloc_wrappers() {
        assert_eq!(encode(b"a b", EncodeSet::Full), "a%20b");
        assert_eq!(decode(b"a%20b"), "a b");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Decoding an encode-set encoding always round-trips exactly.
        #[test]
        fn roundtrip(input in proptest::collection::vec(any::<u8>(), 0..256)) {
            let mut enc = [0u8; 1024];
            let n = percent_encode_into(&input, EncodeSet::Full, &mut enc).unwrap();
            let mut dec = [0u8; 1024];
            let m = percent_decode_into(&enc[..n], &mut dec).unwrap();
            prop_assert_eq!(&dec[..m], &input[..]);
        }

        /// Encoded output contains only unreserved chars or `%` followed by
        /// two hex digits.
        #[test]
        fn encoded_is_valid(input in proptest::collection::vec(any::<u8>(), 0..256)) {
            let mut enc = [0u8; 1024];
            let n = percent_encode_into(&input, EncodeSet::Full, &mut enc).unwrap();
            let e = &enc[..n];
            let mut i = 0;
            while i < e.len() {
                if e[i] == b'%' {
                    prop_assert!(i + 2 < e.len());
                    prop_assert!(hex_val(e[i + 1]).is_some());
                    prop_assert!(hex_val(e[i + 2]).is_some());
                    i += 3;
                } else {
                    prop_assert!(is_unreserved(e[i]));
                    i += 1;
                }
            }
        }
    }
}
