#![no_std]
//! `tpt-zero-slug`: ASCII/Latin-1-safe slug generation.
//!
//! Converts arbitrary text into a URL/identifier-safe "slug": lowercase,
//! ASCII letters/digits separated by hyphens, with a set of common
//! Latin-1-Supplement (`0x80..=0xFF`) characters transliterated to their
//! ASCII look-alikes (`é`→`e`, `ß`→`ss`, `&`→`and`, etc.).
//!
//! # Scope
//!
//! This is a **lightweight** transliteration. It does NOT implement full
//! Unicode NFKD normalization (no combining-mark stripping, no multi-script
//! transliteration). Inputs outside the covered Latin-1 range are either
//! dropped or kept as their transliterated ASCII form. If you need full
//! Unicode slugging, use a dedicated crate. The core API is a buffer writer so
//! it works in `#![no_std]` without heap allocation.
//!
//! - [`slugify_into`] — write a slug into a caller-provided byte buffer.
//! - [`SlugWriter`] — incremental buffer-writer for streaming slugification.
//! - [`slugify`] (alloc feature) — returns a heap-allocated `String`.

use tpt_zero_utf8::Utf8Str;

/// Classification of a single input scalar for slugification.
enum Class {
    /// A word character run, emitted as one or two lowercased ASCII bytes.
    Word1(u8),
    Word2(u8, u8),
    /// A separator: collapse to a single `-` between words.
    Sep,
    /// Drop the character entirely.
    Drop,
}

/// Classify a single Unicode scalar for slugification.
#[inline]
fn classify(ch: char) -> Class {
    if ch.is_ascii() {
        let b = ch as u8;
        if b.is_ascii_alphanumeric() {
            return if b.is_ascii_uppercase() {
                Class::Word1(b + 32)
            } else {
                Class::Word1(b)
            };
        }
        return if matches!(
            b,
            b' ' | b'\t' | b'\n' | b'\r' | b'.' | b'_' | b'/' | b'\\' | b'-' | b'+' | b','
        ) {
            Class::Sep
        } else {
            Class::Drop
        };
    }
    match ch as u32 {
        0x00C0 | 0x00E0 => Class::Word1(b'a'),
        0x00C1 | 0x00E1 => Class::Word1(b'a'),
        0x00C2 | 0x00E2 => Class::Word1(b'a'),
        0x00C3 | 0x00E3 => Class::Word1(b'a'),
        0x00C4 | 0x00E4 => Class::Word2(b'a', b'e'),
        0x00C5 | 0x00E5 => Class::Word1(b'a'),
        0x00C6 | 0x00E6 => Class::Word2(b'a', b'e'),
        0x00C7 | 0x00E7 => Class::Word1(b'c'),
        0x00C8 | 0x00E8 => Class::Word1(b'e'),
        0x00C9 | 0x00E9 => Class::Word1(b'e'),
        0x00CA | 0x00EA => Class::Word1(b'e'),
        0x00CB | 0x00EB => Class::Word1(b'e'),
        0x00CC | 0x00EC => Class::Word1(b'i'),
        0x00CD | 0x00ED => Class::Word1(b'i'),
        0x00CE | 0x00EE => Class::Word1(b'i'),
        0x00CF | 0x00EF => Class::Word1(b'i'),
        0x00D0 | 0x00F0 => Class::Word1(b'd'),
        0x00D1 | 0x00F1 => Class::Word1(b'n'),
        0x00D2 | 0x00F2 => Class::Word1(b'o'),
        0x00D3 | 0x00F3 => Class::Word1(b'o'),
        0x00D4 | 0x00F4 => Class::Word1(b'o'),
        0x00D5 | 0x00F5 => Class::Word1(b'o'),
        0x00D6 | 0x00F6 => Class::Word2(b'o', b'e'),
        0x00D8 | 0x00F8 => Class::Word1(b'o'),
        0x00D9 | 0x00F9 => Class::Word1(b'u'),
        0x00DA | 0x00FA => Class::Word1(b'u'),
        0x00DB | 0x00FB => Class::Word1(b'u'),
        0x00DC | 0x00FC => Class::Word2(b'u', b'e'),
        0x00DD | 0x00FD => Class::Word1(b'y'),
        0x00DE | 0x00FE => Class::Word2(b't', b'h'),
        0x00DF => Class::Word2(b's', b's'),
        0x00FF => Class::Word1(b'y'),
        _ => Class::Drop,
    }
}

/// A buffer-writer that builds a slug incrementally. Words (runs of
/// alphanumeric/transliterable characters) are separated by a single hyphen,
/// with no leading or trailing hyphen.
pub struct SlugWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
    in_run: bool,
    after_word: bool,
}

impl<'a> SlugWriter<'a> {
    /// Create a writer that appends to `buf`.
    #[inline]
    pub fn new(buf: &'a mut [u8]) -> Self {
        SlugWriter {
            buf,
            len: 0,
            in_run: false,
            after_word: false,
        }
    }

    /// Number of bytes written so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether nothing has been been written.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Borrow the written slug bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Feed one `char` of input into the slugifier.
    #[inline]
    pub fn push_char(&mut self, ch: char) {
        match classify(ch) {
            Class::Word1(b) => {
                self.emit_seq(&[b]);
            }
            Class::Word2(a, b) => {
                self.emit_seq(&[a, b]);
            }
            Class::Sep => {
                // End the current word run; the next run will be hyphenated.
                self.in_run = false;
            }
            Class::Drop => {}
        }
    }

    /// Emit a word sequence. If this begins a new run after a completed word
    /// run, a separating hyphen is inserted first.
    fn emit_seq(&mut self, seq: &[u8]) {
        if !self.in_run {
            if self.after_word {
                if self.len + seq.len() + 1 > self.buf.len() {
                    return;
                }
                self.buf[self.len] = b'-';
                self.len += 1;
            } else if self.len + seq.len() > self.buf.len() {
                return;
            }
        } else if self.len + seq.len() > self.buf.len() {
            return;
        }
        self.buf[self.len..self.len + seq.len()].copy_from_slice(seq);
        self.len += seq.len();
        self.in_run = true;
        self.after_word = true;
    }

    /// Feed a whole string slice.
    #[inline]
    pub fn push_str(&mut self, s: &Utf8Str) {
        for (_, ch) in s.char_indices() {
            self.push_char(ch);
        }
    }

    /// Consume the writer and return the written slug slice with the buffer's
    /// original lifetime (releasing the mutable borrow on `buf`).
    #[inline]
    pub fn finish(self) -> &'a [u8] {
        let SlugWriter { buf, len, .. } = self;
        &buf[..len]
    }
}

impl<'a> core::fmt::Write for SlugWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let u = unsafe { Utf8Str::from_bytes_unchecked(s.as_bytes()) };
        self.push_str(u);
        Ok(())
    }
}

/// Slugify `input` into `buf`, returning a slice of the written bytes.
///
/// Returns the empty slice (length 0) if `buf` is too small to hold even the
/// first word. The result is always a valid ASCII slug charset
/// (`[a-z0-9-]`) with no leading/trailing hyphen.
#[inline]
pub fn slugify_into<'a>(input: &Utf8Str, buf: &'a mut [u8]) -> &'a [u8] {
    let mut w = SlugWriter::new(buf);
    w.push_str(input);
    w.finish()
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    extern crate alloc;
    use super::*;
    use alloc::string::String;

    /// Slugify `input`, returning a heap-allocated `String`.
    pub fn slugify(input: &Utf8Str) -> String {
        let mut out = String::with_capacity(input.len() * 2 + 1);
        {
            let mut tmp = [0u8; 512];
            let slice = slugify_into(input, &mut tmp);
            out.push_str(core::str::from_utf8(slice).unwrap_or(""));
        }
        out
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::slugify;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_ascii() {
        let u = tpt_zero_utf8::from_bytes(b"Hello World").unwrap();
        let mut buf = [0u8; 64];
        assert_eq!(slugify_into(u, &mut buf), b"hello-world");
    }

    #[test]
    fn latin1_translit() {
        let s = "Café Münchën Über ß";
        let u = unsafe { Utf8Str::from_bytes_unchecked(s.as_bytes()) };
        let mut buf = [0u8; 64];
        let got = slugify_into(u, &mut buf);
        assert_eq!(core::str::from_utf8(got).unwrap(), "cafe-muenchen-ueber-ss");
    }

    #[test]
    fn punctuation_dropped() {
        let u = tpt_zero_utf8::from_bytes(b"foo!!!bar").unwrap();
        let mut buf = [0u8; 64];
        assert_eq!(slugify_into(u, &mut buf), b"foobar");
    }

    #[test]
    fn leading_trailing_sep() {
        let u = tpt_zero_utf8::from_bytes(b"---hello---world---").unwrap();
        let mut buf = [0u8; 64];
        assert_eq!(slugify_into(u, &mut buf), b"hello-world");
    }

    #[test]
    fn write_trait() {
        use core::fmt::Write;
        let mut buf = [0u8; 64];
        let mut w = SlugWriter::new(&mut buf);
        write!(w, "Hello, Wörld!").unwrap();
        assert_eq!(core::str::from_utf8(w.as_bytes()).unwrap(), "hello-woerld");
    }

    #[test]
    fn edge_empty() {
        let u = tpt_zero_utf8::from_bytes(b"!@#$").unwrap();
        let mut buf = [0u8; 64];
        assert_eq!(slugify_into(u, &mut buf), b"");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn alloc_slugify() {
        let s = "The Quick Brown Fox";
        let u = unsafe { Utf8Str::from_bytes_unchecked(s.as_bytes()) };
        assert_eq!(slugify(u), "the-quick-brown-fox");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Slugify never panics and always yields a valid slug charset.
        #[test]
        fn always_valid_charset(s in ".*") {
            let u = unsafe { Utf8Str::from_bytes_unchecked(s.as_bytes()) };
            let mut buf = [0u8; 1024];
            let out = slugify_into(u, &mut buf);
            for &b in out {
                prop_assert!(b.is_ascii_alphanumeric() || b == b'-');
            }
            if !out.is_empty() {
                prop_assert_ne!(out[0], b'-');
                prop_assert_ne!(out[out.len() - 1], b'-');
            }
            for w in out.windows(2) {
                prop_assert!(!(w[0] == b'-' && w[1] == b'-'));
            }
        }
    }
}
