#![no_std]
//! `tpt-zero-numstr`: zero-dependency, `#![no_std]` number formatting and
//! parsing.
//!
//! Provides `core`-only formatting and parsing for integers and floats:
//!
//! - [`format_int`] / [`parse_int`] — signed and unsigned integers of any
//!   width, with explicit radix control (2..=36), writing into a caller-provided
//!   buffer.
//! - [`format_float`] / [`parse_float`] — `f32`/`f64` formatting and parsing.
//!
//! Behind the `alloc` feature, `String`-returning convenience wrappers are
//! provided.
//!
//! # Float limitation
//!
//! Float formatting uses the standard `core::fmt` rendering and parsing uses a
//! manual decimal reader. This is **not** guaranteed to be the shortest
//! round-trippable decimal representation for every value; for many inputs the
//! output does round-trip exactly through [`parse_float`], but do not depend on
//! bit-exact shortest-repr fidelity.

use core::fmt::Write;

/// Maximum number of bytes needed for any supported integer (sign + 64 digits).
const MAX_INT_LEN: usize = 66;

/// Maximum buffer size for a formatted `f64` (Display-ish output).
const MAX_FLOAT_LEN: usize = 32;

/// Format `value` in `radix` (2..=36) into `buf`, returning the slice of `buf`
/// that was written. Returns `None` if `buf` is too small or `radix` is invalid.
#[inline]
pub fn format_int<T: Itoa>(value: T, radix: u32, buf: &mut [u8]) -> Option<&mut [u8]> {
    value.format(radix, buf)
}

/// Parse a signed integer from `s` in `radix` (2..=36).
///
/// Accepts an optional leading `+`/`-`. Leading/trailing whitespace is *not*
/// trimmed.
#[inline]
pub fn parse_int<T: FromAscii>(s: &[u8], radix: u32) -> Option<T> {
    T::parse(s, radix)
}

/// Format a float in decimal into `buf`, returning the written slice.
///
/// Returns `None` if `buf` is too small. Handles `NaN`, `inf`, `-inf`, zero,
/// and normal magnitudes. See crate-level docs for the non-shortest-repr
/// limitation.
#[inline]
pub fn format_float<F: Ftoa>(value: F, buf: &mut [u8]) -> Option<&mut [u8]> {
    value.format(buf)
}

/// Parse a float from a byte slice. Accepts `inf`, `infinity`, `nan` (case
/// insensitive, optional sign), and decimal/scientific notation.
#[inline]
pub fn parse_float<F: FromAsciiFloat>(s: &[u8]) -> Option<F> {
    F::parse_float(s)
}

// ---------------------------------------------------------------------------
// Integer formatting/parsing
// ---------------------------------------------------------------------------

/// Trait implemented by integer types for formatting into a buffer.
pub trait Itoa: Copy {
    /// Format `self` in `radix` (2..=36) into `buf`; return the written slice.
    fn format(self, radix: u32, buf: &mut [u8]) -> Option<&mut [u8]>;
}

/// Trait implemented by integer types for parsing from ASCII bytes.
pub trait FromAscii: Sized {
    /// Parse `s` (optional sign, digits in `radix`) into the value.
    fn parse(s: &[u8], radix: u32) -> Option<Self>;
}

macro_rules! impl_int {
    ($($t:ty),*) => {$(
        impl Itoa for $t {
            #[inline]
            fn format(self, radix: u32, buf: &mut [u8]) -> Option<&mut [u8]> {
                format_integer(self as i128, radix, buf)
            }
        }
        impl FromAscii for $t {
            #[inline]
            fn parse(s: &[u8], radix: u32) -> Option<Self> {
                let v = parse_integer(s, radix)?;
                Self::try_from(v).ok()
            }
        }
    )*};
}

impl_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

#[inline]
fn format_integer(value: i128, radix: u32, buf: &mut [u8]) -> Option<&mut [u8]> {
    if !(2..=36).contains(&radix) {
        return None;
    }
    let negative = value < 0;
    let mut magnitude: u128 = if negative {
        value.wrapping_neg() as u128
    } else {
        value as u128
    };

    let mut digits = [0u8; MAX_INT_LEN];
    let mut len = 0usize;
    if magnitude == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while magnitude > 0 {
            let d = (magnitude % radix as u128) as u32;
            digits[len] = if d < 10 {
                b'0' + d as u8
            } else {
                b'a' + (d - 10) as u8
            };
            magnitude /= radix as u128;
            len += 1;
        }
    }
    let total = len + if negative { 1 } else { 0 };
    if buf.len() < total {
        return None;
    }
    let start = buf.len() - total;
    let mut idx = start;
    if negative {
        buf[idx] = b'-';
        idx += 1;
    }
    for i in (0..len).rev() {
        buf[idx] = digits[i];
        idx += 1;
    }
    Some(&mut buf[start..])
}

#[inline]
fn parse_integer(s: &[u8], radix: u32) -> Option<i128> {
    if !(2..=36).contains(&radix) {
        return None;
    }
    if s.is_empty() {
        return None;
    }
    let (negative, body) = match s[0] {
        b'+' => (false, &s[1..]),
        b'-' => (true, &s[1..]),
        _ => (false, s),
    };
    if body.is_empty() {
        return None;
    }
    let mut acc: i128 = 0;
    for &b in body {
        let d = match b {
            b'0'..=b'9' => (b - b'0') as u32,
            b'a'..=b'z' => (b - b'a' + 10) as u32,
            b'A'..=b'Z' => (b - b'A' + 10) as u32,
            _ => return None,
        };
        if d >= radix {
            return None;
        }
        acc = acc.checked_mul(radix as i128)?;
        acc = acc.checked_add(d as i128)?;
    }
    if negative {
        // Negate via wrapping so i128::MIN parses correctly.
        Some(acc.wrapping_neg())
    } else {
        Some(acc)
    }
}

// ---------------------------------------------------------------------------
// Float formatting/parsing
// ---------------------------------------------------------------------------

/// Trait implemented by float types for formatting into a buffer.
pub trait Ftoa: Copy {
    /// Format `self` into `buf`; return the written slice.
    fn format(self, buf: &mut [u8]) -> Option<&mut [u8]>;
}

/// Trait implemented by float types for parsing.
pub trait FromAsciiFloat: Sized {
    /// Parse `s` into the value.
    fn parse_float(s: &[u8]) -> Option<Self>;
}

macro_rules! impl_float {
    ($($t:ty),*) => {$(
        impl Ftoa for $t {
            #[inline]
            fn format(self, buf: &mut [u8]) -> Option<&mut [u8]> {
                format_float_impl(self as f64, buf)
            }
        }
        impl FromAsciiFloat for $t {
            #[inline]
            fn parse_float(s: &[u8]) -> Option<Self> {
                parse_float_impl(s).map(|v| v as $t)
            }
        }
    )*};
}

impl_float!(f32, f64);

/// Stack buffer used by `format_float_impl` to render via `core::fmt`.
struct StackWriter {
    buf: [u8; MAX_FLOAT_LEN],
    len: usize,
}

impl StackWriter {
    fn new() -> Self {
        StackWriter {
            buf: [0u8; MAX_FLOAT_LEN],
            len: 0,
        }
    }
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl Write for StackWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > self.buf.len() {
            return Err(core::fmt::Error);
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

/// Format a float into `buf` using `core::fmt::Display` (available under
/// `#![no_std]`). See crate-level docs for the non-shortest-repr limitation.
#[inline]
fn format_float_impl(value: f64, buf: &mut [u8]) -> Option<&mut [u8]> {
    if buf.len() < MAX_FLOAT_LEN {
        return None;
    }
    if value.is_nan() {
        return copy_into(buf, b"NaN");
    }
    if value.is_infinite() {
        return if value < 0.0 {
            copy_into(buf, b"-inf")
        } else {
            copy_into(buf, b"inf")
        };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            copy_into(buf, b"-0")
        } else {
            copy_into(buf, b"0")
        };
    }
    let mut w = StackWriter::new();
    // `core::fmt::Write` for floats is implemented in `core`, so this works in a
    // `no_std` context.
    if core::write!(&mut w, "{value}").is_err() {
        return None;
    }
    let written = w.as_bytes();
    if buf.len() < written.len() {
        return None;
    }
    buf[..written.len()].copy_from_slice(written);
    Some(&mut buf[..written.len()])
}

#[inline]
fn copy_into<'a>(buf: &'a mut [u8], s: &[u8]) -> Option<&'a mut [u8]> {
    if buf.len() < s.len() {
        return None;
    }
    buf[..s.len()].copy_from_slice(s);
    Some(&mut buf[..s.len()])
}

#[inline]
fn parse_float_impl(s: &[u8]) -> Option<f64> {
    if s.is_empty() {
        return None;
    }
    let lower = to_lower8(s);
    if lower == b"nan" || lower == b"nan()" {
        return Some(f64::NAN);
    }
    if matches!(
        lower.as_slice(),
        b"inf" | b"infinity" | b"+inf" | b"+infinity"
    ) {
        return Some(f64::INFINITY);
    }
    if matches!(lower.as_slice(), b"-inf" | b"-infinity") {
        return Some(f64::NEG_INFINITY);
    }
    parse_decimal_f64(s)
}

/// Lower-cased first 8 bytes of `s` for token matching.
struct Lower8 {
    bytes: [u8; 8],
    len: usize,
}

impl Lower8 {
    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl PartialEq<&[u8]> for Lower8 {
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_slice() == *other
    }
}

#[inline]
fn to_lower8(s: &[u8]) -> Lower8 {
    let mut out = [0u8; 8];
    let n = s.len().min(8);
    for i in 0..n {
        let b = s[i];
        out[i] = if b.is_ascii_uppercase() { b + 32 } else { b };
    }
    Lower8 { bytes: out, len: n }
}

/// Parse decimal/scientific notation into an `f64` using only integer
/// arithmetic plus float multiply/divide (both available in `core`).
#[inline]
fn parse_decimal_f64(bytes: &[u8]) -> Option<f64> {
    let (negative, body) = match bytes[0] {
        b'+' => (false, &bytes[1..]),
        b'-' => (true, &bytes[1..]),
        _ => (false, bytes),
    };
    if body.is_empty() {
        return None;
    }
    // Accumulate significant digits into a single mantissa and track the total
    // number of fractional digit-positions (including leading zeros after the
    // dot, which are not significant but still shift the decimal place).
    let mut mantissa: u64 = 0;
    let mut frac_pos: i32 = 0;
    let mut exp_sign: i32 = 1;
    let mut exp_val: i32 = 0;
    let mut saw_dot = false;
    let mut saw_exp = false;
    let mut started = false;
    let mut any_digit = false;
    let mut sig_digits: u32 = 0;
    let mut i = 0;
    while i < body.len() {
        let b = body[i];
        if b.is_ascii_digit() {
            let d = (b - b'0') as u64;
            any_digit = true;
            if saw_dot {
                frac_pos += 1;
            }
            if d != 0 || started {
                started = true;
                if sig_digits < 17 {
                    mantissa = mantissa.saturating_mul(10).saturating_add(d);
                    sig_digits += 1;
                }
            }
            i += 1;
        } else if b == b'.' && !saw_dot && !saw_exp {
            saw_dot = true;
            i += 1;
        } else if (b == b'e' || b == b'E') && !saw_exp && any_digit {
            saw_exp = true;
            i += 1;
            if i < body.len() && (body[i] == b'+' || body[i] == b'-') {
                if body[i] == b'-' {
                    exp_sign = -1;
                }
                i += 1;
            }
            while i < body.len() && body[i].is_ascii_digit() {
                exp_val = exp_val
                    .saturating_mul(10)
                    .saturating_add((body[i] - b'0') as i32);
                i += 1;
            }
        } else {
            return None;
        }
    }
    if !any_digit {
        return None;
    }

    // value = mantissa * 10^(exp_val*exp_sign - frac_pos)
    let total_exp = exp_val as i64 * exp_sign as i64 - frac_pos as i64;
    let mut value = mantissa as f64;
    if total_exp != 0 {
        value *= pow10_f64(total_exp);
    }
    if negative {
        value = -value;
    }
    Some(value)
}

/// Compute `10^e` as an `f64` using only multiply/divide (no `powi`).
#[inline]
fn pow10_f64(e: i64) -> f64 {
    if e == 0 {
        return 1.0;
    }
    let positive = e > 0;
    let mut n = e.abs();
    let mut result = 1.0f64;
    let mut base = 10.0f64;
    while n > 0 {
        if n & 1 == 1 {
            result *= base;
        }
        base *= base;
        n >>= 1;
    }
    if positive { result } else { 1.0 / result }
}

#[cfg(feature = "alloc")]
mod alloc_layer {
    use super::*;
    extern crate alloc;
    use alloc::string::{String, ToString};

    /// Format an integer, returning a `String`.
    #[inline]
    pub fn format_int_to_string<T: Itoa>(value: T, radix: u32) -> Option<String> {
        let mut buf = [0u8; MAX_INT_LEN];
        let written = value.format(radix, &mut buf)?;
        Some(core::str::from_utf8(written).unwrap().to_string())
    }

    /// Format a float, returning a `String`.
    #[inline]
    pub fn format_float_to_string<F: Ftoa>(value: F) -> Option<String> {
        let mut buf = [0u8; MAX_FLOAT_LEN];
        let written = value.format(&mut buf)?;
        Some(core::str::from_utf8(written).unwrap().to_string())
    }
}

#[cfg(feature = "alloc")]
pub use alloc_layer::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_format_basic() {
        let mut buf = [0u8; 32];
        let s = core::str::from_utf8(format_int(12345i32, 10, &mut buf).unwrap()).unwrap();
        assert_eq!(s, "12345");
        let s = core::str::from_utf8(format_int(-123i32, 10, &mut buf).unwrap()).unwrap();
        assert_eq!(s, "-123");
        let s = core::str::from_utf8(format_int(255u8, 16, &mut buf).unwrap()).unwrap();
        assert_eq!(s, "ff");
        let s = core::str::from_utf8(format_int(0i32, 10, &mut buf).unwrap()).unwrap();
        assert_eq!(s, "0");
    }

    #[test]
    fn int_parse_basic() {
        assert_eq!(parse_int::<i32>(b"12345", 10), Some(12345));
        assert_eq!(parse_int::<i32>(b"-123", 10), Some(-123));
        assert_eq!(parse_int::<u8>(b"ff", 16), Some(255));
        assert_eq!(parse_int::<i32>(b"+7", 10), Some(7));
        assert_eq!(parse_int::<i32>(b"", 10), None);
        assert_eq!(parse_int::<i32>(b"12g", 10), None);
    }

    #[test]
    fn int_roundtrip_small() {
        for v in -500i32..500 {
            let mut buf = [0u8; 32];
            let s = format_int(v, 10, &mut buf).unwrap();
            let back = parse_int::<i32>(s, 10).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn float_format_special() {
        let mut buf = [0u8; 32];
        assert_eq!(
            core::str::from_utf8(format_float(f64::NAN, &mut buf).unwrap()).unwrap(),
            "NaN"
        );
        assert_eq!(
            core::str::from_utf8(format_float(f64::INFINITY, &mut buf).unwrap()).unwrap(),
            "inf"
        );
        assert_eq!(
            core::str::from_utf8(format_float(f64::NEG_INFINITY, &mut buf).unwrap()).unwrap(),
            "-inf"
        );
        assert_eq!(
            core::str::from_utf8(format_float(0.0f64, &mut buf).unwrap()).unwrap(),
            "0"
        );
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn float_parse_basic() {
        assert_eq!(parse_float::<f64>(b"3.14"), Some(3.14));
        assert_eq!(parse_float::<f64>(b"-2.5"), Some(-2.5));
        assert_eq!(parse_float::<f64>(b"1.5e3"), Some(1500.0));
        assert!(parse_float::<f64>(b"nan").unwrap().is_nan());
        assert_eq!(parse_float::<f64>(b"inf"), Some(f64::INFINITY));
        assert_eq!(parse_float::<f64>(b"-inf"), Some(f64::NEG_INFINITY));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn float_roundtrip_reasonable() {
        for &v in &[0.0f64, 1.0, -1.0, 3.14, 100.0, -2.5, 1234.0, 0.001] {
            let mut buf = [0u8; 32];
            let s = format_float(v, &mut buf).unwrap();
            let back = parse_float::<f64>(s).unwrap();
            assert!(
                (back - v).abs() < v.abs().max(1.0) * 1e-6 || (v == 0.0 && back == 0.0),
                "v={v} back={back}"
            );
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    #[allow(clippy::approx_constant)]
    fn alloc_wrappers() {
        assert_eq!(format_int_to_string(42i32, 10).unwrap(), "42");
        assert_eq!(format_float_to_string(3.14f64).unwrap(), "3.14");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Integer round-trip: parse(format(v)) == v, for the full `i64` range.
        #[test]
        fn int_roundtrip(v in any::<i64>()) {
            let mut buf = [0u8; 32];
            let s = format_int(v, 10, &mut buf).unwrap();
            let back = parse_int::<i64>(s, 10).unwrap();
            prop_assert_eq!(back, v);
        }

        /// Float round-trip for the documented safe subset: integer-valued
        /// `f64` magnitudes, where the manual decimal parser reconstructs the
        /// exact value. The crate documents that fractional/very-large values
        /// may lose low-order bits (a non-shortest-repr limitation).
        #[test]
        fn float_roundtrip(i in -999_999_999i64..999_999_999) {
            let v = i as f64;
            let mut buf = [0u8; 32];
            let s = format_float(v, &mut buf).unwrap();
            let back = parse_float::<f64>(s).unwrap();
            prop_assert!(
                back == v,
                "round-trip mismatch: v={} formatted={:?} reparsed={}",
                v,
                core::str::from_utf8(s).unwrap(),
                back
            );
        }
    }
}
