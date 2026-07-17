#![no_std]
//! `tpt-zero-utf8`: zero-dependency, `#![no_std]` UTF-8 utilities.
//!
//! Provides:
//! - [`from_bytes`] — validate a slice as well-formed UTF-8.
//! - [`Utf8Str::from_bytes_unchecked`] — wrap a slice that is *known* to be valid UTF-8.
//! - [`next_char_boundary`] / [`prev_char_boundary`] — find the nearest
//!   char-start byte at or after / at or before an arbitrary index.
//! - [`CharIndices`] — a safe scalar (Unicode code point) iterator that never
//!   panics and never yields a replacement char for malformed input; instead it
//!   yields each invalid byte as its own "scalar" of value `0xE000 | byte` so the
//!   caller can decide what to do.
//! - [`encode_char`] — encode a single `char` into a provided buffer.
//!
//! Everything here is `#![no_std]` and depends only on `core`.

/// A validated UTF-8 string slice.
///
/// This is a transparent wrapper around `&[u8]` that guarantees the bytes are
/// well-formed UTF-8. Construct one with [`from_bytes`].
///
/// Note: `Utf8Str` is unsized (`[u8]` tail) and therefore is only ever handled
/// by reference. It intentionally does not implement `Clone`/`Copy`.
#[repr(transparent)]
pub struct Utf8Str {
    bytes: [u8],
}

impl Utf8Str {
    /// The underlying bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// View this UTF-8 string as a `str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: the only way to construct a `Utf8Str` is through `from_bytes`
        // (which validates) or `from_bytes_unchecked` (which shifts the
        // responsibility to the caller).
        unsafe { core::str::from_utf8_unchecked(&self.bytes) }
    }

    /// Iterate over the Unicode scalars (code points) in this string, returning
    /// their `(byte_offset, char)` pairs.
    #[inline]
    pub fn char_indices(&self) -> CharIndices<'_> {
        CharIndices {
            bytes: &self.bytes,
            pos: 0,
        }
    }

    /// Length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Returns `true` if the string contains no bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl core::ops::Deref for Utf8Str {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl core::fmt::Debug for Utf8Str {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), f)
    }
}

/// Validate `bytes` as UTF-8.
///
/// Returns `Ok(&Utf8Str)` if the slice is well-formed UTF-8, or
/// `Err(ValidationError { valid_up_to, error_len })` describing the first
/// invalid sequence.
#[inline]
pub fn from_bytes(bytes: &[u8]) -> Result<&Utf8Str, ValidationError> {
    match core::str::from_utf8(bytes) {
        Ok(_) => {
            // SAFETY: `core::str::from_utf8` returned `Ok`, so `bytes` is valid UTF-8.
            Ok(unsafe { Utf8Str::from_bytes_unchecked(bytes) })
        }
        Err(e) => Err(ValidationError {
            valid_up_to: e.valid_up_to(),
            error_len: e.error_len().map(|l| l as u8),
        }),
    }
}

impl Utf8Str {
    /// Wrap a byte slice as a `Utf8Str` without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must be well-formed UTF-8. Passing invalid UTF-8 makes the
    /// `as_str`/`Deref` methods produce undefined behaviour.
    #[inline]
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Utf8Str {
        // SAFETY: `Utf8Str` is `#[repr(transparent)]` over `[u8]`.
        unsafe { &*(bytes as *const [u8] as *const Utf8Str) }
    }
}

/// Description of the first invalid UTF-8 sequence found by [`from_bytes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValidationError {
    /// Index of the first byte of the invalid sequence.
    pub valid_up_to: usize,
    /// Length of the invalid sequence in bytes, or `None` if it is truncated at
    /// the end of the slice.
    pub error_len: Option<u8>,
}

/// Find the index of the first byte that starts a new code point at or after
/// `index` (clamped to the slice length).
///
/// This is useful for slicing a byte buffer at a boundary that will not split a
/// code point.
#[inline]
pub fn next_char_boundary(bytes: &[u8], mut index: usize) -> usize {
    let len = bytes.len();
    if index >= len {
        return len;
    }
    // Skip continuation bytes (0b10xxxxxx).
    while index < len && (bytes[index] & 0xC0) == 0x80 {
        index += 1;
    }
    index
}

/// Find the index of the first byte of the code point that contains `index`, or
/// the nearest code-point start at or before `index`.
///
/// Returns `0` if `index == 0` or if `index` points into the middle/end of a
/// multi-byte sequence (the result is then the start of that sequence).
#[inline]
pub fn prev_char_boundary(bytes: &[u8], mut index: usize) -> usize {
    let len = bytes.len();
    if index >= len {
        if len == 0 {
            return 0;
        }
        index = len - 1;
    }
    // Step back while the byte *at* `index` is a continuation byte
    // (0b10xxxxxx); stop when we reach a lead byte or the start.
    while index > 0 && (bytes[index] & 0xC0) == 0x80 {
        index -= 1;
    }
    index
}

/// An iterator over the Unicode scalars of a [`Utf8Str`].
///
/// Yields `(byte_offset, char)` pairs. For malformed input it does *not* yield
/// replacement characters; instead it emits each offending byte as a char in the
/// surrogate range `0xE000..=0xDCFF` so callers can detect and handle errors
/// without panicking and without depending on `alloc`.
#[derive(Clone, Debug)]
pub struct CharIndices<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<(usize, char)> {
        let bytes = self.bytes;
        if self.pos >= bytes.len() {
            return None;
        }
        let start = self.pos;
        let (ch, size) = decode_utf8_byte(bytes, start);
        self.pos = start + size;
        Some((start, ch))
    }
}

/// Decode the code point starting at `start`. Returns the `char` (or a surrogate
/// sentinel for invalid bytes) and the number of bytes consumed (always >= 1).
#[inline]
fn decode_utf8_byte(bytes: &[u8], start: usize) -> (char, usize) {
    let b = bytes[start];
    if b < 0x80 {
        // ASCII fast path.
        return (b as char, 1);
    }
    // Determine the width from the lead byte.
    let width = utf8_width(b);
    if width == 0 {
        // Invalid lead byte: emit a sentinel in the surrogate range.
        return (char::from_u32(0xE000 | (b as u32)).unwrap(), 1);
    }
    let len = bytes.len();
    if start + width > len {
        // Truncated sequence: emit sentinel for the lead byte.
        return (char::from_u32(0xE000 | (b as u32)).unwrap(), 1);
    }
    let mut code: u32 = (b & (0x7F >> width)) as u32;
    for i in 1..width {
        let c = bytes[start + i];
        if (c & 0xC0) != 0x80 {
            // Invalid continuation byte: sentinel for the lead byte.
            return (char::from_u32(0xE000 | (b as u32)).unwrap(), 1);
        }
        code = (code << 6) | (c & 0x3F) as u32;
    }
    match char::from_u32(code) {
        Some(ch) => (ch, width),
        None => (char::from_u32(0xE000 | (b as u32)).unwrap(), 1),
    }
}

/// Width in bytes of a UTF-8 sequence led by `b`, or `0` if `b` is not a valid
/// lead byte.
#[inline]
fn utf8_width(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => 0,
    }
}

/// Encode a single `char` into `buf`, returning the number of bytes written.
///
/// Writes at most 4 bytes. Returns `None` if `buf` is too small.
#[inline]
pub fn encode_char(ch: char, buf: &mut [u8]) -> Option<usize> {
    let code = ch as u32;
    let (first, rest): (u8, u32) = match code {
        0..=0x7F => {
            if buf.is_empty() {
                return None;
            }
            buf[0] = code as u8;
            return Some(1);
        }
        0x80..=0x7FF => (0xC0 | ((code >> 6) & 0x1F) as u8, code),
        0x800..=0xFFFF => (0xE0 | ((code >> 12) & 0x0F) as u8, code),
        0x10000..=0x10FFFF => (0xF0 | ((code >> 18) & 0x07) as u8, code),
        _ => return None,
    };
    let width = utf8_width(first);
    if buf.len() < width {
        return None;
    }
    match width {
        2 => {
            buf[0] = first | (rest >> 6) as u8;
            buf[1] = 0x80 | (rest & 0x3F) as u8;
        }
        3 => {
            buf[0] = first | (rest >> 12) as u8;
            buf[1] = 0x80 | ((rest >> 6) & 0x3F) as u8;
            buf[2] = 0x80 | (rest & 0x3F) as u8;
        }
        4 => {
            buf[0] = first | (rest >> 18) as u8;
            buf[1] = 0x80 | ((rest >> 12) & 0x3F) as u8;
            buf[2] = 0x80 | ((rest >> 6) & 0x3F) as u8;
            buf[3] = 0x80 | (rest & 0x3F) as u8;
        }
        _ => unreachable!(),
    }
    Some(width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_ascii() {
        assert!(from_bytes(b"hello").is_ok());
    }

    #[test]
    fn validate_utf8_multibyte() {
        let s = "héllo 世界 🦀";
        assert!(from_bytes(s.as_bytes()).is_ok());
        let v = from_bytes(s.as_bytes()).unwrap();
        assert_eq!(v.as_str(), s);
    }

    #[test]
    fn reject_truncated() {
        let err = from_bytes(&[0xE2, 0x82]).unwrap_err();
        assert_eq!(err.valid_up_to, 0);
    }

    #[test]
    fn reject_invalid_lead() {
        let err = from_bytes(&[0xFE]).unwrap_err();
        assert_eq!(err.valid_up_to, 0);
    }

    #[test]
    fn next_boundary_skips_cont() {
        let s = "é"; // 0xC3 0xA9
        let bytes = s.as_bytes();
        assert_eq!(next_char_boundary(bytes, 1), 2);
        assert_eq!(next_char_boundary(bytes, 2), 2);
    }

    #[test]
    fn prev_boundary() {
        let s = "é";
        let bytes = s.as_bytes();
        assert_eq!(prev_char_boundary(bytes, 1), 0);
        assert_eq!(prev_char_boundary(bytes, 2), 0);
    }

    #[test]
    fn char_indices_ok() {
        let s = "aé🦀";
        let v = from_bytes(s.as_bytes()).unwrap();
        let mut collected = ['\0'; 3];
        for (i, (_, c)) in v.char_indices().enumerate() {
            collected[i] = c;
        }
        assert_eq!(collected, ['a', 'é', '🦀']);
    }

    #[test]
    fn char_indices_invalid_sentinel() {
        // 0xFF is an invalid lead byte; it should yield a surrogate sentinel.
        let v = unsafe { Utf8Str::from_bytes_unchecked(&[0xFF, b'a']) };
        let mut it = v.char_indices();
        let (_, c0) = it.next().unwrap();
        assert!((c0 as u32) >= 0xE000);
        let (_, c1) = it.next().unwrap();
        assert_eq!(c1, 'a');
    }

    #[test]
    fn encode_char_roundtrip() {
        for &ch in &['a', 'é', '世', '🦀'] {
            let mut buf = [0u8; 4];
            let n = encode_char(ch, &mut buf).unwrap();
            let back = from_bytes(&buf[..n]).unwrap();
            let mut cbuf = [0u8; 4];
            let cn = encode_char(ch, &mut cbuf).unwrap();
            let expected = core::str::from_utf8(&cbuf[..cn]).unwrap();
            assert_eq!(back.as_str(), expected);
        }
    }

    #[test]
    fn encode_char_small_buf() {
        let mut buf = [0u8; 1];
        assert_eq!(encode_char('é', &mut buf), None);
    }
}
