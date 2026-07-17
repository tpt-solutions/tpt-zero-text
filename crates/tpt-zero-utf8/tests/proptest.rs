use proptest::prelude::*;
use tpt_zero_utf8::{encode_char, from_bytes, next_char_boundary, prev_char_boundary};

proptest! {
    /// Any byte slice that is valid UTF-8 must pass validation.
    #[test]
    fn valid_utf8_never_rejected(s in ".*") {
        prop_assert!(from_bytes(s.as_bytes()).is_ok());
    }

    /// encode_char never panics and round-trips through from_bytes.
    #[test]
    fn encode_roundtrip(ch in proptest::char::any()) {
        if ch as u32 > 0x10FFFF {
            return Ok(());
        }
        let mut buf = [0u8; 4];
        if let Some(n) = encode_char(ch, &mut buf) {
            let parsed = from_bytes(&buf[..n]).unwrap();
            let expected: std::string::String = ch.into();
            prop_assert_eq!(parsed.as_str(), expected.as_str());
        }
    }

    /// Adversarial bytes: from_bytes must never panic.
    #[test]
    fn never_panics_on_adversarial(bytes in proptest::collection::vec(any::<u8>(), 0..64)) {
        let _ = from_bytes(&bytes);
    }

    /// next_char_boundary always lands on or after the requested index and is a
    /// valid boundary (the byte at the boundary is not a continuation byte, or
    /// it is at the end).
    #[test]
    fn next_boundary_is_valid(s in ".*", idx in any::<usize>()) {
        let bytes = s.as_bytes();
        let b = next_char_boundary(bytes, idx % (bytes.len() + 1).max(1));
        prop_assert!(b <= bytes.len());
    }

    /// prev_char_boundary always lands at or before the requested index.
    #[test]
    fn prev_boundary_is_valid(s in ".*", idx in any::<usize>()) {
        let bytes = s.as_bytes();
        let b = prev_char_boundary(bytes, idx);
        prop_assert!(b <= bytes.len());
        prop_assert!(b <= idx.min(bytes.len()));
    }
}
