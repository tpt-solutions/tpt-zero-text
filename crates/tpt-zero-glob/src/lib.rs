#![no_std]
//! `tpt-zero-glob`: a small, `#![no_std]` glob matcher.
//!
//! Supports the common shell-style wildcards:
//! - `*` — match any sequence of characters (within a single path segment).
//! - `?` — match exactly one character.
//! - `**` — match across path separators (zero or more segments).
//! - `[...]` — a character class. Ranges (`a-z`), negation (`[!...]`), and
//!   individual characters are supported.
//!
//! `{a,b}` alternation is **deferred to v0.2**.
//!
//! - [`matches()`] / [`matches_str`] — one-shot compile-and-match helpers.
//! - [`Pattern::compile`] — parse a glob once and reuse it via [`Pattern::matches`].
//!
//! Matching is byte-oriented (so it works on arbitrary byte slices in
//! `no_std`); for text paths pass `&[u8]` obtained from `str::as_bytes`.
//!
//! The matcher walks the pattern and text bytes directly (no intermediate
//! allocation), so it works without a heap allocator even for arbitrarily
//! long patterns.

/// A compiled glob pattern. Compiling only borrows the pattern bytes — there
/// is no separate AST, so this is essentially free and needs no heap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pattern<'a> {
    bytes: &'a [u8],
}

impl<'a> Pattern<'a> {
    /// Compile a glob pattern from bytes.
    #[inline]
    pub fn compile(pattern: &'a [u8]) -> Pattern<'a> {
        Pattern { bytes: pattern }
    }

    /// Compile a glob pattern from a `&str`.
    #[inline]
    pub fn compile_str(pattern: &'a str) -> Pattern<'a> {
        Pattern {
            bytes: pattern.as_bytes(),
        }
    }

    /// Match `text` (a byte slice) against the compiled pattern.
    #[inline]
    pub fn matches(&self, text: &[u8]) -> bool {
        do_match(self.bytes, text)
    }

    /// Match `text` (a `&str`) against the compiled pattern.
    #[inline]
    pub fn matches_str(&self, text: &str) -> bool {
        do_match(self.bytes, text.as_bytes())
    }
}

/// Convenience: compile `pattern` and match `text` in one call.
#[inline]
pub fn matches(pattern: &[u8], text: &[u8]) -> bool {
    do_match(pattern, text)
}

/// Convenience: compile a `&str` pattern and match a `&str` text in one call.
#[inline]
pub fn matches_str(pattern: &str, text: &str) -> bool {
    do_match(pattern.as_bytes(), text.as_bytes())
}

/// Recursive backtracking matcher: `pat` against `text`, operating directly
/// on the byte slices (no intermediate allocation).
fn do_match(pat: &[u8], txt: &[u8]) -> bool {
    if pat.is_empty() {
        return txt.is_empty();
    }

    // `**` — matches zero or more whole path segments, optionally followed
    // by a `/` and more pattern.
    if pat.len() >= 2 && pat[0] == b'*' && pat[1] == b'*' && (pat.len() == 2 || pat[2] == b'/') {
        let after = &pat[2..];
        let rest: &[u8] = if after.first() == Some(&b'/') {
            &after[1..]
        } else {
            after
        };
        if rest.is_empty() {
            // Nothing follows `**`: it consumes everything remaining.
            return true;
        }
        if do_match(rest, txt) {
            // Zero segments consumed.
            return true;
        }
        let mut i = 0;
        while i < txt.len() {
            if txt[i] == b'/' && do_match(pat, &txt[i + 1..]) {
                return true;
            }
            i += 1;
        }
        return false;
    }

    match pat[0] {
        b'*' => {
            // Matches any run of bytes within this segment (never crosses `/`).
            let mut i = 0;
            loop {
                if do_match(&pat[1..], &txt[i..]) {
                    return true;
                }
                if i >= txt.len() || txt[i] == b'/' {
                    return false;
                }
                i += 1;
            }
        }
        b'?' => !txt.is_empty() && txt[0] != b'/' && do_match(&pat[1..], &txt[1..]),
        b'[' => {
            let (matched, consumed) = class_match(pat, txt.first().copied());
            matched && do_match(&pat[consumed..], &txt[1..])
        }
        b'/' => !txt.is_empty() && txt[0] == b'/' && do_match(&pat[1..], &txt[1..]),
        c => !txt.is_empty() && txt[0] == c && do_match(&pat[1..], &txt[1..]),
    }
}

/// Parse the `[...]` class at `pat[0]` (`pat[0]` must be `[`) and, if `byte`
/// is `Some`, test it for membership. Returns `(matched, consumed)` where
/// `consumed` is the number of pattern bytes making up the class (from `[`
/// through the closing `]`, inclusive).
fn class_match(pat: &[u8], byte: Option<u8>) -> (bool, usize) {
    let mut i = 1;
    let negated = if i < pat.len() && (pat[i] == b'!' || pat[i] == b'^') {
        i += 1;
        true
    } else {
        false
    };
    let mut hit = false;
    while i < pat.len() && pat[i] != b']' {
        let a = pat[i];
        if i + 2 < pat.len() && pat[i + 1] == b'-' && pat[i + 2] != b']' {
            let z = pat[i + 2];
            if let Some(b) = byte {
                if a <= b && b <= z {
                    hit = true;
                }
            }
            i += 3;
        } else {
            if let Some(b) = byte {
                if a == b {
                    hit = true;
                }
            }
            i += 1;
        }
    }
    if i < pat.len() {
        i += 1; // consume ']'
    }
    let matched = match byte {
        Some(_) => hit != negated,
        None => false,
    };
    (matched, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal() {
        assert!(matches_str("foo", "foo"));
        assert!(!matches_str("foo", "bar"));
    }

    #[test]
    fn star() {
        assert!(matches_str("f*", "foo"));
        assert!(matches_str("*o", "foo"));
        assert!(matches_str("*", "anything"));
        assert!(matches_str("f*o", "foo"));
        assert!(!matches_str("a*", "ba"));
    }

    #[test]
    fn qmark() {
        assert!(matches_str("f?o", "foo"));
        assert!(!matches_str("f?o", "fo"));
        assert!(!matches_str("f?o", "fooo"));
    }

    #[test]
    fn class_basic() {
        assert!(matches_str("[abc]", "b"));
        assert!(!matches_str("[abc]", "d"));
        assert!(matches_str("[a-z]", "m"));
        assert!(!matches_str("[a-z]", "M"));
    }

    #[test]
    fn class_negated() {
        assert!(matches_str("[!0-9]", "a"));
        assert!(!matches_str("[!0-9]", "5"));
    }

    #[test]
    fn segments() {
        assert!(matches_str("src/*.rs", "src/lib.rs"));
        assert!(!matches_str("src/*.rs", "src/a/b.rs"));
        assert!(matches_str("a/b/c", "a/b/c"));
        assert!(!matches_str("a/b", "a/b/c"));
    }

    #[test]
    fn starstar() {
        assert!(matches_str("src/**/*.rs", "src/lib.rs"));
        assert!(matches_str("src/**/*.rs", "src/a/b/c.rs"));
        assert!(matches_str("**", "a/b/c/d"));
        assert!(matches_str("a/**/z", "a/b/c/z"));
        assert!(!matches_str("a/**/z", "a/y"));
    }

    #[test]
    fn mixed() {
        assert!(matches_str("**/[a-z]*.txt", "x/y/foo.txt"));
        assert!(matches_str("images/*.png", "images/a.png"));
    }

    #[test]
    fn compiled_pattern_reuse() {
        let p = Pattern::compile_str("build/**/*.o");
        assert!(p.matches_str("build/obj/foo.o"));
        assert!(p.matches_str("build/foo.o"));
        assert!(!p.matches_str("build/foo.c"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    extern crate std;
    use std::{format, string::String};

    proptest! {
        /// A literal (wildcard-free) pattern always matches exactly that
        /// text, and never matches a strictly longer text.
        #[test]
        fn literal_roundtrip(s in "[a-zA-Z0-9_./]{0,32}") {
            prop_assert!(matches_str(&s, &s));
            let longer: String = format!("{s}x");
            prop_assert!(!matches_str(&s, &longer));
        }

        /// `*.*` matches any non-empty text containing at least one `.`
        /// (and no `/`, since `*` never crosses a segment boundary).
        #[test]
        fn star_dot_star(s in "[a-zA-Z0-9_.]{0,32}") {
            let got = matches_str("*.*", &s);
            let want = s.contains('.') && !s.is_empty();
            prop_assert_eq!(got, want);
        }

        /// `**` matches every possible text (it spans all segments).
        #[test]
        fn starstar_matches_all(s in "[a-zA-Z0-9_./]{0,32}") {
            prop_assert!(matches_str("**", &s));
        }

        /// `[a-z]` matches any single lowercase ASCII letter and rejects
        /// anything else.
        #[test]
        fn class_lower(b in any::<u8>()) {
            let s = [b];
            let text = core::str::from_utf8(&s);
            if let Ok(text) = text {
                let got = matches_str("[a-z]", text);
                let want = b.is_ascii_lowercase();
                prop_assert_eq!(got, want);
            }
        }
    }
}
