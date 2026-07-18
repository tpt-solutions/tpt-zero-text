#![no_std]
//! `out-zero-arraystring`: a `String`-like buffer backed by a fixed-size array.
//!
//! [`ArrayString<const N>`] stores up to `N` UTF-8 bytes inline (on top of
//! [`out_zero_arrayvec::ArrayVec<u8, N>`]). All mutating operations are
//! UTF-8-safe: a `push`/`push_str` that would split a code point or introduce
//! invalid UTF-8 is rejected with a [`Utf8Error`] instead of panicking.
//!
//! Derefs to `str`, implements [`core::fmt::Write`], and is `no_std`-friendly.

use core::fmt;
use core::ops::Deref;
use out_zero_arrayvec::ArrayVec;
use tpt_zero_utf8::{Utf8Str, from_bytes};

/// Error returned when a push would overflow capacity or break UTF-8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Utf8Error {
    /// The push would exceed the buffer's `N`-byte capacity.
    Overflow,
    /// The pushed bytes would produce invalid UTF-8 (e.g. a split code point).
    InvalidUtf8,
}

/// A `String`-like type backed by `[u8; N]`.
pub struct ArrayString<const N: usize> {
    buf: ArrayVec<u8, N>,
}

impl<const N: usize> ArrayString<N> {
    /// Create an empty `ArrayString`.
    pub const fn new() -> Self {
        ArrayString {
            buf: ArrayVec::new(),
        }
    }

    /// Current length in bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether the string is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Whether the string has reached its byte capacity.
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.buf.is_full()
    }

    /// Total byte capacity.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Push a single character, returning a [`Utf8Error`] if it would overflow
    /// or (it never does for a valid `char`) break UTF-8.
    #[inline]
    pub fn push(&mut self, ch: char) -> Result<(), Utf8Error> {
        let mut bytes = [0u8; 4];
        let n = tpt_zero_utf8::encode_char(ch, &mut bytes).ok_or(Utf8Error::InvalidUtf8)?;
        if self.buf.len() + n > N {
            return Err(Utf8Error::Overflow);
        }
        for &b in &bytes[..n] {
            // SAFETY: capacity checked above.
            self.buf.push(b).ok();
        }
        Ok(())
    }

    /// Push a string slice, returning a [`Utf8Error`] on overflow or if it would
    /// leave the buffer with invalid UTF-8 (a code point split across the join).
    #[inline]
    pub fn push_str(&mut self, s: &str) -> Result<(), Utf8Error> {
        let bytes = s.as_bytes();
        if self.buf.len() + bytes.len() > N {
            return Err(Utf8Error::Overflow);
        }
        // The buffer is always valid UTF-8 ending on a code-point boundary.
        // Appending `bytes` stays valid UTF-8 unless `bytes` *begins* mid code
        // point (a leading continuation byte), which can only happen for input
        // that is itself not a valid standalone `str` boundary join.
        if !bytes.is_empty() && (bytes[0] & 0xC0) == 0x80 {
            return Err(Utf8Error::InvalidUtf8);
        }
        for &b in bytes {
            // SAFETY: capacity checked above.
            self.buf.push(b).ok();
        }
        Ok(())
    }

    /// Remove and return the last character, if any.
    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        if self.buf.is_empty() {
            return None;
        }
        // Step back over a UTF-8 sequence (lead byte has 0b11xxxxxx or is ASCII).
        let mut end = self.buf.len();
        loop {
            end -= 1;
            let b = self.buf[end];
            if (b & 0xC0) != 0x80 {
                // Found the lead byte.
                let slice = &self.buf[end..self.buf.len()];
                let ch = from_bytes(slice)
                    .ok()
                    .and_then(|u| u.as_str().chars().next());
                // Truncate to `end`.
                while self.buf.len() > end {
                    self.buf.pop();
                }
                return ch;
            }
            if end == 0 {
                // Defensive: should never happen for valid buffer.
                while !self.buf.is_empty() {
                    self.buf.pop();
                }
                return None;
            }
        }
    }

    /// Clear the string.
    #[inline]
    pub fn clear(&mut self) {
        while self.buf.pop().is_some() {}
    }

    /// Borrow the contents as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// View the contents as a `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: every push is UTF-8-validated, so the buffer is always valid.
        unsafe { core::str::from_utf8_unchecked(&self.buf) }
    }
}

impl<const N: usize> Default for ArrayString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for ArrayString<N> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Display for ArrayString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for ArrayString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> fmt::Write for ArrayString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s).map_err(|_| fmt::Error)
    }
}

impl<const N: usize> From<&Utf8Str> for ArrayString<N> {
    fn from(s: &Utf8Str) -> Self {
        let mut out = ArrayString::new();
        let _ = out.push_str(s.as_str());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_ascii() {
        let mut s: ArrayString<8> = ArrayString::new();
        assert!(s.push('h').is_ok());
        assert!(s.push('i').is_ok());
        assert_eq!(s.as_str(), "hi");
        assert_eq!(s.pop(), Some('i'));
        assert_eq!(s.as_str(), "h");
    }

    #[test]
    fn push_multibyte() {
        let mut s: ArrayString<8> = ArrayString::new();
        assert!(s.push('é').is_ok());
        assert!(s.push_str("🦀").is_ok());
        assert_eq!(s.as_str(), "é🦀");
    }

    #[test]
    fn overflow_is_error() {
        let mut s: ArrayString<3> = ArrayString::new();
        assert!(s.push_str("abc").is_ok());
        assert_eq!(s.push('d'), Err(Utf8Error::Overflow));
        assert_eq!(s.push_str("xy"), Err(Utf8Error::Overflow));
        assert_eq!(s.as_str(), "abc");
    }

    #[test]
    fn write_trait() {
        use core::fmt::Write;
        let mut s: ArrayString<16> = ArrayString::new();
        write!(s, "n={}", 42).unwrap();
        assert_eq!(s.as_str(), "n=42");
    }

    #[test]
    fn pop_multibyte() {
        let mut s: ArrayString<8> = ArrayString::new();
        s.push_str("a🦀").unwrap();
        assert_eq!(s.pop(), Some('🦀'));
        assert_eq!(s.as_str(), "a");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Pushing arbitrary valid `str` content never produces invalid UTF-8.
        #[test]
        fn push_str_never_invalid_utf8(s in ".*") {
            let mut a: ArrayString<64> = ArrayString::new();
            // Always succeeds (or errors on overflow), never leaves invalid UTF-8.
            let _ = a.push_str(&s);
            prop_assert!(core::str::from_utf8(a.as_bytes()).is_ok());
        }

        /// Pushing valid `char`s never produces invalid UTF-8.
        #[test]
        fn push_chars_never_invalid_utf8(c in any::<char>()) {
            let mut a: ArrayString<16> = ArrayString::new();
            let _ = a.push(c);
            prop_assert!(core::str::from_utf8(a.as_bytes()).is_ok());
        }
    }
}
