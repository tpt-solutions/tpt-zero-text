#![no_std]
//! `tpt-zero-vec`: minimal `f32` vector types for `#![no_std]`.
//!
//! Provides [`Vec2`], [`Vec3`], and [`Vec4`], each a `#[repr(C)]` tuple of
//! `f32`s with the usual arithmetic operators (`+`, `-`, `*`, `/` against
//! scalars and same-type vectors). Length/normalization use
//! [`tpt-zero-fast-math`](https://docs.rs/tpt-zero-fast-math) (`fast_sqrt` / `fast_inv_sqrt`), so they are
//! dependency-free and allocation-free but carry the same `~1e-3` accuracy
//! characteristics.
//!
//! # Scope
//!
//! `f32`-only in v0.1. `f64` vectors are deferred to v0.2. There is no
//! `alloc` API in v0.1 (vectors are fixed-size stack values).

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use tpt_zero_fast_math::{fast_inv_sqrt, fast_sqrt};

macro_rules! impl_vec {
    ($name:ident, $n:literal, $($field:ident),+) => {
        /// A fixed-size `f32` vector.
        #[derive(Clone, Copy, Debug, Default, PartialEq)]
        #[repr(C)]
        pub struct $name {
            $(pub $field: f32),+
        }

        impl $name {
            /// Construct a new vector from its components.
            #[inline]
            pub const fn new($($field: f32),+) -> Self {
                Self { $($field),+ }
            }

            /// The squared length (`dot(self, self)`). Cheaper than [`Self::length`].
            #[inline]
            pub fn length_sq(self) -> f32 {
                self.dot(self)
            }

            /// The Euclidean length. Uses the fast square-root approximation.
            #[inline]
            pub fn length(self) -> f32 {
                fast_sqrt(self.length_sq())
            }

            /// Dot (inner) product with `other`.
            #[inline]
            pub fn dot(self, other: Self) -> f32 {
                let mut acc = 0.0f32;
                $(acc += self.$field * other.$field;)+
                acc
            }

            /// Returns `self` scaled to unit length. Returns `self` unchanged if
            /// the length is zero (to avoid a divide-by-zero).
            #[inline]
            pub fn normalize(self) -> Self {
                let len_sq = self.length_sq();
                if len_sq == 0.0 {
                    return self;
                }
                self * fast_inv_sqrt(len_sq)
            }

            /// Element-wise minimum.
            #[inline]
            pub fn min(self, other: Self) -> Self {
                Self { $($field: self.$field.min(other.$field)),+ }
            }

            /// Element-wise maximum.
            #[inline]
            pub fn max(self, other: Self) -> Self {
                Self { $($field: self.$field.max(other.$field)),+ }
            }
        }

        impl Add for $name {
            type Output = Self;
            #[inline]
            fn add(self, other: Self) -> Self {
                Self { $($field: self.$field + other.$field),+ }
            }
        }

        impl AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                $(self.$field += other.$field;)+
            }
        }

        impl Sub for $name {
            type Output = Self;
            #[inline]
            fn sub(self, other: Self) -> Self {
                Self { $($field: self.$field - other.$field),+ }
            }
        }

        impl SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                $(self.$field -= other.$field;)+
            }
        }

        impl Neg for $name {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self { $($field: -self.$field),+ }
            }
        }

        impl Mul<f32> for $name {
            type Output = Self;
            #[inline]
            fn mul(self, s: f32) -> Self {
                Self { $($field: self.$field * s),+ }
            }
        }

        impl Mul<$name> for f32 {
            type Output = $name;
            #[inline]
            fn mul(self, v: $name) -> $name {
                v * self
            }
        }

        impl MulAssign<f32> for $name {
            #[inline]
            fn mul_assign(&mut self, s: f32) {
                $(self.$field *= s;)+
            }
        }

        impl Div<f32> for $name {
            type Output = Self;
            #[inline]
            fn div(self, s: f32) -> Self {
                Self { $($field: self.$field / s),+ }
            }
        }

        impl DivAssign<f32> for $name {
            #[inline]
            fn div_assign(&mut self, s: f32) {
                $(self.$field /= s;)+
            }
        }
    };
}

impl_vec!(Vec2, 2, x, y);
impl_vec!(Vec3, 3, x, y, z);
impl_vec!(Vec4, 4, x, y, z, w);

impl core::ops::Index<usize> for Vec2 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Vec2 index out of range"),
        }
    }
}

impl core::ops::Index<usize> for Vec3 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Vec3 index out of range"),
        }
    }
}

impl core::ops::Index<usize> for Vec4 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Vec4 index out of range"),
        }
    }
}

impl Vec3 {
    /// Cross product. Only defined for 3-vectors.
    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Truncate to a [`Vec2`] (drops `z`).
    #[inline]
    pub fn xy(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Extend to a [`Vec4`] with the given `w`.
    #[inline]
    pub fn extend(self, w: f32) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, w)
    }
}

impl Vec4 {
    /// Truncate to a [`Vec3`] (drops `w`).
    #[inline]
    pub fn xyz(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn scalar_mul_div() {
        let a = Vec2::new(2.0, -4.0);
        assert_eq!(a * 2.0, Vec2::new(4.0, -8.0));
        assert_eq!(a / 2.0, Vec2::new(1.0, -2.0));
        assert_eq!(3.0 * a, Vec2::new(6.0, -12.0));
    }

    #[test]
    fn dot_length() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(a.dot(b), 0.0);
        assert!((a.length() - 1.0).abs() < 1e-3);
        assert!(((a * 3.0).length() - 3.0).abs() < 1e-3);
    }

    #[test]
    fn normalize_unit() {
        let a = Vec3::new(0.0, 5.0, 0.0);
        let n = a.normalize();
        assert!((n.length() - 1.0).abs() < 1e-2);
        assert!((n.y - 1.0).abs() < 1e-2);
    }

    #[test]
    fn normalize_zero_is_identity() {
        assert_eq!(
            Vec3::new(0.0, 0.0, 0.0).normalize(),
            Vec3::new(0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn cross() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(x.cross(y), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn conversions() {
        let v3 = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v3.xy(), Vec2::new(1.0, 2.0));
        assert_eq!(v3.extend(4.0), Vec4::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(Vec4::new(1.0, 2.0, 3.0, 4.0).xyz(), v3);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// `dot(a, a)` equals `length_sq(a)` and is non-negative.
        #[test]
        fn dot_self_is_len_sq(x in -1e3f32..1e3, y in -1e3f32..1e3, z in -1e3f32..1e3) {
            let a = Vec3::new(x, y, z);
            prop_assert!((a.dot(a) - a.length_sq()).abs() < 1e-2 * a.length_sq().abs().max(1.0));
            prop_assert!(a.dot(a) >= 0.0);
        }

        /// `normalize` produces a unit vector (within the fast-math tolerance)
        /// for any non-zero input.
        #[test]
        fn normalize_unit_len(x in -1e3f32..1e3, y in -1e3f32..1e3, z in -1e3f32..1e3) {
            let a = Vec3::new(x, y, z);
            if a.length_sq() > 1e-6 {
                let n = a.normalize();
                prop_assert!((n.length() - 1.0).abs() < 1e-2, "len={}", n.length());
            }
        }

        /// Vector addition is commutative.
        #[test]
        fn add_commutative(x in -1e3f32..1e3, y in -1e3f32..1e3, u in -1e3f32..1e3, v in -1e3f32..1e3) {
            let a = Vec2::new(x, y);
            let b = Vec2::new(u, v);
            prop_assert_eq!(a + b, b + a);
        }

        /// Cross product is anti-commutative.
        #[test]
        fn cross_anticommute(x in -1e3f32..1e3, y in -1e3f32..1e3, z in -1e3f32..1e3,
                             u in -1e3f32..1e3, v in -1e3f32..1e3, w in -1e3f32..1e3) {
            let a = Vec3::new(x, y, z);
            let b = Vec3::new(u, v, w);
            prop_assert_eq!(a.cross(b), -b.cross(a));
        }
    }
}
