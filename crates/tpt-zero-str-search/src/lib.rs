#![no_std]
//! `tpt-zero-str-search`: zero-dependency, `#![no_std]` substring search.
//!
//! - [`find_byte`] / [`rfind_byte`] — memchr-style single-byte search.
//! - [`find`] / [`rfind`] — Boyer-Moore-Horspool substring search over byte
//!   slices, returning byte offsets.
//! - [`Finder`] — a reusable Boyer-Moore-Horspool searcher that precomputes the
//!   bad-character shift table once and can be applied repeatedly.

/// Find the first index of `needle` in `haystack`, or `None`.
#[inline]
pub fn find_byte(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Find the last index of `needle` in `haystack`, or `None`.
#[inline]
pub fn rfind_byte(needle: u8, haystack: &[u8]) -> Option<usize> {
    haystack.iter().rposition(|&b| b == needle)
}

/// Find the first index at which `needle` occurs in `haystack`, or `None`.
#[inline]
pub fn find(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    Finder::new(needle).find_in(haystack)
}

/// Find the last index at which `needle` occurs in `haystack`, or `None`.
#[inline]
pub fn rfind(needle: &[u8], haystack: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(haystack.len());
    }
    Finder::new(needle).rfind_in(haystack)
}

/// A Boyer-Moore-Horspool searcher with a precomputed bad-character table.
#[derive(Clone)]
pub struct Finder {
    needle: [u8; 64],
    needle_len: usize,
    shifts: [usize; 256],
}

impl Finder {
    /// Build a searcher for `needle`. Panics if `needle` is longer than 64 bytes.
    #[inline]
    pub fn new(needle: &[u8]) -> Finder {
        assert!(needle.len() <= 64, "needle too long (max 64 bytes)");
        let mut n = [0u8; 64];
        n[..needle.len()].copy_from_slice(needle);
        let m = needle.len();
        let mut shifts = [m; 256];
        if m > 1 {
            // For each position i in 0..m-1, the shift for byte needle[i] is
            // m - 1 - i (last occurrence wins because we overwrite).
            for (i, &b) in needle.iter().take(m - 1).enumerate() {
                shifts[b as usize] = m - 1 - i;
            }
        }
        Finder {
            needle: n,
            needle_len: m,
            shifts,
        }
    }

    /// Find the first occurrence of the needle in `haystack`.
    #[inline]
    pub fn find_in(&self, haystack: &[u8]) -> Option<usize> {
        let m = self.needle_len;
        let n = haystack.len();
        if m == 0 {
            return Some(0);
        }
        if n < m {
            return None;
        }
        let last = m - 1;
        let mut i = last;
        while i < n {
            let mut j = last;
            let mut k = i;
            while j < m && self.needle[j] == haystack[k] {
                if j == 0 {
                    return Some(k);
                }
                j -= 1;
                k -= 1;
            }
            i += self.shifts[haystack[i] as usize];
        }
        None
    }

    /// Find the last occurrence of the needle in `haystack`.
    #[inline]
    pub fn rfind_in(&self, haystack: &[u8]) -> Option<usize> {
        let m = self.needle_len;
        let n = haystack.len();
        if m == 0 {
            return Some(n);
        }
        if n < m {
            return None;
        }
        let mut i = n - m + 1;
        while i > 0 {
            i -= 1;
            if &haystack[i..i + m] == self.needle_bytes() {
                return Some(i);
            }
        }
        None
    }

    #[inline]
    fn needle_bytes(&self) -> &[u8] {
        &self.needle[..self.needle_len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_basic() {
        assert_eq!(find_byte(b'a', b"banana"), Some(1));
        assert_eq!(rfind_byte(b'a', b"banana"), Some(5));
        assert_eq!(find_byte(b'z', b"banana"), None);
    }

    #[test]
    fn substring_basic() {
        assert_eq!(find(b"ana", b"banana"), Some(1));
        assert_eq!(rfind(b"ana", b"banana"), Some(3));
        assert_eq!(find(b"xyz", b"banana"), None);
        assert_eq!(find(b"", b"banana"), Some(0));
        assert_eq!(find(b"banana", b"banana"), Some(0));
    }

    #[test]
    fn substring_longer_haystack() {
        let h = b"The quick brown fox jumps over the lazy dog";
        assert_eq!(find(b"lazy", h), Some(35));
        assert_eq!(rfind(b"the", h), Some(31));
        assert_eq!(find(b"dog", h), Some(40));
    }

    #[test]
    fn needle_longer_than_haystack() {
        assert_eq!(find(b"abcd", b"abc"), None);
    }

    #[test]
    fn repeated_needle() {
        // Exercises bad-character shifts with repeating bytes.
        assert_eq!(find(b"aa", b"baaabaa"), Some(1));
        assert_eq!(rfind(b"aa", b"baaabaa"), Some(5));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// find must agree with std str::find on a byte level.
        #[test]
        fn oracle_vs_std(hay in ".*", needle in "[x-z]{0,64}") {
            let hay = hay.as_bytes();
            let ndl = needle.as_bytes();
            let got = find(ndl, hay).map(|i| &hay[i..i + ndl.len()]);
            let want = if ndl.is_empty() {
                Some(&hay[..0])
            } else {
                hay.windows(ndl.len()).position(|w| w == ndl).map(|i| &hay[i..i + ndl.len()])
            };
            prop_assert_eq!(got, want);
        }

        /// rfind must agree with a naive reverse scan.
        #[test]
        fn rfind_oracle(hay in ".*", needle in "[x-z]{0,64}") {
            let hay = hay.as_bytes();
            let ndl = needle.as_bytes();
            let got = rfind(ndl, hay);
            let want = if ndl.is_empty() {
                Some(hay.len())
            } else {
                let mut found = None;
                if ndl.len() <= hay.len() {
                    for i in 0..=hay.len() - ndl.len() {
                        if &hay[i..i + ndl.len()] == ndl {
                            found = Some(i);
                        }
                    }
                }
                found
            };
            prop_assert_eq!(got, want);
        }

        /// find_byte must agree with Iterator::position.
        #[test]
        fn find_byte_oracle(hay in ".*", b in any::<u8>()) {
            let hay = hay.as_bytes();
            prop_assert_eq!(find_byte(b, hay), hay.iter().position(|&x| x == b));
        }
    }
}
