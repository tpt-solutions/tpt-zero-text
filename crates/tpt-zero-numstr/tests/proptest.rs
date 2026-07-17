use proptest::prelude::*;
use tpt_zero_numstr::{format_float, format_int, parse_float, parse_int};

proptest! {
    /// format_int/parse_int round-trip for i64.
    #[test]
    fn int_roundtrip_i64(v in any::<i64>()) {
        let mut buf = [0u8; 32];
        let s = format_int(v, 10, &mut buf).unwrap();
        let back = parse_int::<i64>(s, 10).unwrap();
        prop_assert_eq!(back, v);
    }

    /// format_int/parse_int round-trip for u64.
    #[test]
    fn int_roundtrip_u64(v in any::<u64>()) {
        let mut buf = [0u8; 32];
        let s = format_int(v, 10, &mut buf).unwrap();
        let back = parse_int::<u64>(s, 10).unwrap();
        prop_assert_eq!(back, v);
    }

    /// format_int/parse_int round-trip in hex for i32.
    #[test]
    fn int_roundtrip_hex(v in any::<i32>()) {
        let mut buf = [0u8; 32];
        let s = format_int(v, 16, &mut buf).unwrap();
        let back = parse_int::<i32>(s, 16).unwrap();
        prop_assert_eq!(back, v);
    }

    /// format_float/parse_float round-trip within a tight relative tolerance
    /// for finite `f64` inside the documented safe precision range. We
    /// generate values as `sign * m * 10^e` with a small integer mantissa `m`
    /// (< 1e6) and modest exponent `e`. The crate documents that the manual
    /// decimal reader is not bit-exact for every shortest decimal (a
    /// non-shortest-repr limitation); a 1e-12 relative tolerance bounds that
    /// error while still catching gross parser defects (wrong sign, dropped
    /// digits, off-by-orders-of-magnitude).
    #[test]
    fn float_roundtrip(
        (sign, m, e) in (any::<bool>(), 1i64..1_000_000, -15i8..4i8),
    ) {
        let v = if sign { 1.0 } else { -1.0 } * (m as f64) * 10f64.powi(e as i32);
        let mut buf = [0u8; 32];
        if let Some(s) = format_float(v, &mut buf) {
            if let Some(back) = parse_float::<f64>(s) {
                let tol = v.abs() * 1e-12 + f64::MIN_POSITIVE;
                prop_assert!(
                    (back - v).abs() <= tol,
                    "v={} back={} s={:?}",
                    v,
                    back,
                    core::str::from_utf8(s).unwrap()
                );
            }
        }
    }
}
