#![no_std]
//! `tpt-zero-fast-math`: zero-dependency, `#![no_std]` approximate math.
//!
//! Provides fast, deterministic approximations:
//! - `fast_sqrt` / `fast_inv_sqrt`: via bit-hack + Newton-Raphson.
//! - `sin`/`cos`/`tan`/`asin`/`acos`: via a small const
//!   lookup table with linear interpolation.
//!
//! # Accuracy
//!
//! These are **approximations** intended for games, DSP, and similar workloads.
//! Typical absolute error is around `1e-4` for `sqrt`/`inv_sqrt` and `1e-4`
//! radians for the trig functions. Do **not** use them where bit-exact
//! `std` math is required.

/// Integer square root (floor of ?n), exact, no floats required.
#[inline]
pub fn isqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Approximate `sqrt(x)` for `x >= 0`. Uses the fast inverse square root
/// plus one refinement. Error is ~1e-4 relative.
#[inline]
pub fn fast_sqrt(x: f32) -> f32 {
    if x < 0.0 {
        return f32::NAN;
    }
    if x == 0.0 {
        return 0.0;
    }
    let inv = fast_inv_sqrt(x);
    if inv.is_infinite() {
        return 0.0;
    }
    let g = 1.0 / inv;
    let r = 0.5 * (g + x / g);
    if r < 0.0 { -r } else { r }
}

/// Approximate `1 / sqrt(x)` for `x > 0`. Uses the IEEE-754 bit trick plus
/// one Newton-Raphson refinement. Error is ~1e-4 relative.
#[inline]
pub fn fast_inv_sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return f32::INFINITY;
    }
    let bits = x.to_bits();
    // Magic constant for f32 inv-sqrt: 0x5F375A86.
    let guess_bits = 0x5F375A86u32.wrapping_sub(bits >> 1);
    let mut guess = f32::from_bits(guess_bits);
    // One Newton-Raphson step: g = g * (1.5 - 0.5 * x * g * g).
    guess = guess * (1.5 - 0.5 * x * guess * guess);
    guess
}

// ---------------------------------------------------------------------------
// Trig via const lookup table (Taylor-free, deterministic).
// ---------------------------------------------------------------------------

/// Number of table entries for a quarter period (0..=PI/2).
const TRIG_N: usize = 512;
const TRIG_STEP: f32 = core::f32::consts::FRAC_PI_2 / (TRIG_N as f32);

/// Precomputed `(sin, cos)` over [0, PI/2] inclusive, built at compile time
/// with a stable Euler integrator from the exact (0, 1) seed. This avoids
/// factorial blow-up and is deterministic.
const fn build_trig_table() -> ([f32; TRIG_N + 1], [f32; TRIG_N + 1]) {
    let mut sint = [0.0f32; TRIG_N + 1];
    let mut cost = [0.0f32; TRIG_N + 1];
    // Seed at x=0.
    let mut s = 0.0f32;
    let mut c = 1.0f32;
    let mut i = 0;
    while i <= TRIG_N {
        sint[i] = s;
        cost[i] = c;
        let h = TRIG_STEP;
        let ns = s + h * c;
        let nc = c - h * s;
        // Keep the unit-circle invariant stable without a const `sqrt`: one
        // Newton step on the reciprocal of the norm.
        let r2 = ns * ns + nc * nc;
        let f = 1.5 - 0.5 * r2;
        s = ns * f;
        c = nc * f;
        i += 1;
    }
    (sint, cost)
}

static TRIG: ([f32; TRIG_N + 1], [f32; TRIG_N + 1]) = build_trig_table();
static SIN_TABLE: [f32; TRIG_N + 1] = TRIG.0;
static COS_TABLE: [f32; TRIG_N + 1] = TRIG.1;
/// For `asin` we store the angle associated with each sin sample.
static ASIN_TABLE: [f32; TRIG_N + 1] = {
    let mut table = [0.0f32; TRIG_N + 1];
    let mut i = 0;
    while i <= TRIG_N {
        table[i] = (i as f32) * TRIG_STEP;
        i += 1;
    }
    table
};

/// Linearly interpolate `sin` for an angle `x` in [0, PI/2].
#[inline]
fn sample_sin(x: f32) -> f32 {
    let t = (x / TRIG_STEP).clamp(0.0, TRIG_N as f32);
    let i = t as usize;
    if i >= TRIG_N {
        return SIN_TABLE[TRIG_N];
    }
    let frac = t - i as f32;
    SIN_TABLE[i] * (1.0 - frac) + SIN_TABLE[i + 1] * frac
}

/// Linearly interpolate `cos` for an angle `x` in [0, PI/2].
#[inline]
fn sample_cos(x: f32) -> f32 {
    let t = (x / TRIG_STEP).clamp(0.0, TRIG_N as f32);
    let i = t as usize;
    if i >= TRIG_N {
        return COS_TABLE[TRIG_N];
    }
    let frac = t - i as f32;
    COS_TABLE[i] * (1.0 - frac) + COS_TABLE[i + 1] * frac
}

/// Approximate `sin(x)` (radians). Error ~1e-4.
#[inline]
pub fn sin(x: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let mut a = x % tau;
    if a < 0.0 {
        a += tau;
    }
    let pi = core::f32::consts::PI;
    let pi2 = core::f32::consts::FRAC_PI_2;
    if a <= pi2 {
        sample_sin(a)
    } else if a <= pi {
        sample_sin(pi - a)
    } else if a <= 3.0 * pi2 {
        -sample_sin(a - pi)
    } else {
        -sample_sin(2.0 * pi - a)
    }
}

/// Approximate `cos(x)` (radians). Error ~1e-4.
#[inline]
pub fn cos(x: f32) -> f32 {
    let tau = core::f32::consts::TAU;
    let mut a = x % tau;
    if a < 0.0 {
        a += tau;
    }
    let pi = core::f32::consts::PI;
    let pi2 = core::f32::consts::FRAC_PI_2;
    if a <= pi2 {
        sample_cos(a)
    } else if a <= pi {
        -sample_cos(pi - a)
    } else if a <= 3.0 * pi2 {
        -sample_cos(a - pi)
    } else {
        sample_cos(2.0 * pi - a)
    }
}

/// Approximate `tan(x)` (radians). Error ~1e-4. May be large near odd
/// multiples of PI/2 (poles), as with any finite approximation.
#[inline]
pub fn tan(x: f32) -> f32 {
    let c = cos(x);
    if c.abs() < 1e-6 {
        return if sin(x) >= 0.0 {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        };
    }
    sin(x) / c
}

/// Approximate `asin(x)` for `x` in [-1, 1]. Error ~1e-4.
#[inline]
pub fn asin(x: f32) -> f32 {
    if !(-1.0..=1.0).contains(&x) {
        return f32::NAN;
    }
    let y = x.abs();
    // SIN_TABLE is sorted ascending; binary search for y then lerp ASIN_TABLE.
    if y <= SIN_TABLE[0] {
        return x.signum() * ASIN_TABLE[0];
    }
    if y >= SIN_TABLE[TRIG_N] {
        return x.signum() * ASIN_TABLE[TRIG_N];
    }
    let mut lo = 0usize;
    let mut hi = TRIG_N;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if SIN_TABLE[mid] <= y {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let frac = (y - SIN_TABLE[lo]) / (SIN_TABLE[hi] - SIN_TABLE[lo]);
    let angle = ASIN_TABLE[lo] * (1.0 - frac) + ASIN_TABLE[hi] * frac;
    x.signum() * angle
}

/// Approximate `acos(x)` for `x` in [-1, 1]. Error ~1e-4.
#[inline]
pub fn acos(x: f32) -> f32 {
    if !(-1.0..=1.0).contains(&x) {
        return f32::NAN;
    }
    core::f32::consts::FRAC_PI_2 - asin(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_known() {
        assert!((fast_sqrt(4.0) - 2.0).abs() < 1e-3);
        assert!((fast_sqrt(2.0) - core::f32::consts::SQRT_2).abs() < 1e-3);
        assert!((fast_sqrt(100.0) - 10.0).abs() < 1e-2);
        assert_eq!(fast_sqrt(0.0), 0.0);
        assert!(fast_sqrt(-1.0).is_nan());
    }

    #[test]
    fn inv_sqrt_known() {
        assert!((fast_inv_sqrt(4.0) - 0.5).abs() < 1e-3);
        assert!((fast_inv_sqrt(2.0) - (1.0 / core::f32::consts::SQRT_2)).abs() < 1e-3);
        assert!(fast_inv_sqrt(0.0).is_infinite());
    }

    #[test]
    fn trig_known() {
        assert!((sin(0.0)).abs() < 1e-4);
        assert!((cos(0.0) - 1.0).abs() < 1e-4);
        assert!((sin(core::f32::consts::FRAC_PI_2) - 1.0).abs() < 1e-3);
        assert!((cos(core::f32::consts::PI) + 1.0).abs() < 1e-3);
        assert!((tan(core::f32::consts::FRAC_PI_4) - 1.0).abs() < 2e-3);
    }

    #[test]
    fn asin_acos_known() {
        assert!((asin(0.0)).abs() < 1e-4);
        assert!((asin(1.0) - core::f32::consts::FRAC_PI_2).abs() < 1e-3);
        assert!((acos(1.0)).abs() < 1e-3);
        assert!((acos(0.0) - core::f32::consts::FRAC_PI_2).abs() < 1e-3);
    }

    #[test]
    fn isqrt_basic() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(15), 3);
        assert_eq!(isqrt(16), 4);
        assert_eq!(isqrt(17), 4);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// fast_sqrt(x)^2 approximates x within ~1e-2 relative.
        #[test]
        fn sqrt_error(x in 0.0f32..=1e6) {
            let r = fast_sqrt(x);
            prop_assert!(r >= 0.0);
            let err = (r * r - x).abs() / x.max(1.0);
            prop_assert!(err < 1e-2, "sqrt err too large: {}", err);
        }

        /// inv_sqrt round-trips: 1/inv_sqrt(x) ~= sqrt(x).
        #[test]
        fn inv_sqrt_roundtrip(x in 1e-3f32..=1e6) {
            let inv = fast_inv_sqrt(x);
            let back = 1.0 / inv;
            let err = (back * back - x).abs() / x;
            prop_assert!(err < 1e-2, "inv_sqrt err: {}", err);
        }

        /// sin^2 + cos^2 ~= 1 within ~1e-3.
        #[test]
        fn sin_cos_identity(th in -10.0f32..=10.0) {
            let s = sin(th);
            let c = cos(th);
            prop_assert!((s * s + c * c - 1.0).abs() < 1e-3, "ident {}", s * s + c * c);
        }

        /// asin(sin(x)) ~= x on the principal domain.
        #[test]
        fn asin_sin(x in -1.5f32..=1.5) {
            let s = sin(x);
            if s.abs() <= 1.0 {
                let a = asin(s);
                prop_assert!((a - x).abs() < 1e-2, "asin(sin({}))={}", x, a);
            }
        }
    }
}
