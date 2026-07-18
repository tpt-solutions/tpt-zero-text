#![no_std]
//! `tpt-zero-quat`: minimal `f32` quaternion math for `#![no_std]`.
//!
//! Provides [`Quat`], a unit-norm quaternion used for 3D rotations. The
//! Hamilton product, axis-angle construction, spherical-linear interpolation
//! (`slerp`), and vector rotation (`rotate_vec3`) are all provided. Trigonometry
//! and square roots use [`tpt-zero-fast-math`](https://docs.rs/tpt-zero-fast-math), so the crate is dependency-free
//! and allocation-free but carries the same `~1e-3` accuracy characteristics.
//!
//! # Convention
//!
//! Quaternions are stored as `(x, y, z, w)` where `(x, y, z)` is the vector part
//! and `w` is the scalar (real) part. Rotations are applied as `q * v * q^-1`.
//!
//! # Scope (v0.1)
//!
//! `f32`-only. No `Mat3`/`Mat4` conversion in v0.1 (see `tpt-zero-matrix`).

use tpt_zero_fast_math::{acos, cos, fast_sqrt, sin};
use tpt_zero_vec::Vec3;

/// A quaternion `(x, y, z, w)` with `w` the scalar (real) part.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Quat {
    /// Vector part x.
    pub x: f32,
    /// Vector part y.
    pub y: f32,
    /// Vector part z.
    pub z: f32,
    /// Scalar (real) part.
    pub w: f32,
}

impl Quat {
    /// Construct a quaternion from its components `(x, y, z, w)`.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quat { x, y, z, w }
    }

    /// The identity (no-rotation) quaternion `(0, 0, 0, 1)`.
    #[inline]
    pub const fn identity() -> Self {
        Quat::new(0.0, 0.0, 0.0, 1.0)
    }

    /// The squared norm (`x*x + y*y + z*z + w*w`).
    #[inline]
    pub fn norm_sq(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// The norm (length). Uses the fast square-root approximation.
    #[inline]
    pub fn norm(self) -> f32 {
        fast_sqrt(self.norm_sq())
    }

    /// Returns `self` normalized to unit length. Returns `self` unchanged if the
    /// norm is zero (to avoid a divide-by-zero).
    #[inline]
    pub fn normalize(self) -> Self {
        let n = self.norm();
        if n == 0.0 {
            return self;
        }
        Quat::new(self.x / n, self.y / n, self.z / n, self.w / n)
    }

    /// Hamilton product `self * rhs`.
    #[inline]
    pub fn hamilton(self, rhs: Quat) -> Quat {
        Quat::new(
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        )
    }

    /// The conjugate `q*` (negates the vector part).
    #[inline]
    pub fn conjugate(self) -> Quat {
        Quat::new(-self.x, -self.y, -self.z, self.w)
    }

    /// The inverse `q^-1`. For a unit quaternion this is the conjugate.
    /// Returns `None` if the quaternion is (near) zero length.
    #[inline]
    pub fn invert(self) -> Option<Quat> {
        let n = self.norm_sq();
        if n < 1e-12 {
            return None;
        }
        let inv = 1.0 / n;
        Some(Quat::new(
            -self.x * inv,
            -self.y * inv,
            -self.z * inv,
            self.w * inv,
        ))
    }

    /// Build a unit quaternion representing a rotation of `angle_rad` radians
    /// about the normalized `axis`.
    #[inline]
    pub fn from_axis_angle(axis: Vec3, angle_rad: f32) -> Quat {
        let axis = axis.normalize();
        let half = angle_rad * 0.5;
        let s = sin(half);
        Quat::new(axis.x * s, axis.y * s, axis.z * s, cos(half))
    }

    /// Rotate `v` by this (unit) quaternion, returning `q * v * q^-1`.
    #[inline]
    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let q = self.normalize();
        let vq = Quat::new(v.x, v.y, v.z, 0.0);
        let result = q.hamilton(vq).hamilton(q.conjugate());
        Vec3::new(result.x, result.y, result.z)
    }
}

impl core::ops::Mul for Quat {
    type Output = Quat;
    #[inline]
    fn mul(self, rhs: Quat) -> Quat {
        self.hamilton(rhs)
    }
}

impl core::ops::MulAssign for Quat {
    #[inline]
    fn mul_assign(&mut self, rhs: Quat) {
        *self = self.hamilton(rhs);
    }
}

/// Spherical-linear interpolation between two unit quaternions.
///
/// `t == 0` returns `a`, `t == 1` returns `b`. Both inputs should be
/// approximately unit length; they are normalized internally.
#[inline]
pub fn slerp(a: Quat, b: Quat, t: f32) -> Quat {
    let a = a.normalize();
    let mut b = b.normalize();

    // Compute the cosine of the angle between the two quaternions.
    let mut dot = a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;

    // If the dot product is negative, the quaternions are on opposite hemispheres
    // of the 4D hypersphere; flip one so we interpolate the shorter arc.
    if dot < 0.0 {
        b = Quat::new(-b.x, -b.y, -b.z, -b.w);
        dot = -dot;
    }

    // If the inputs are nearly identical, fall back to normalized linear
    // interpolation to avoid dividing by (almost) zero.
    if dot > 0.9995 {
        let r = Quat::new(
            a.x + (b.x - a.x) * t,
            a.y + (b.y - a.y) * t,
            a.z + (b.z - a.z) * t,
            a.w + (b.w - a.w) * t,
        );
        return r.normalize();
    }

    let theta_0 = acos(dot);
    let theta = theta_0 * t;
    let sin_theta = sin(theta);
    let sin_theta_0 = sin(theta_0);

    let s0 = cos(theta) - dot * sin_theta / sin_theta_0;
    let s1 = sin_theta / sin_theta_0;

    Quat::new(
        a.x * s0 + b.x * s1,
        a.y * s0 + b.y * s1,
        a.z * s0 + b.z * s1,
        a.w * s0 + b.w * s1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_no_rotation() {
        let q = Quat::identity();
        let v = Vec3::new(1.0, 2.0, 3.0);
        let r = q.rotate_vec3(v);
        assert!((r.x - 1.0).abs() < 1e-3);
        assert!((r.y - 2.0).abs() < 1e-3);
        assert!((r.z - 3.0).abs() < 1e-3);
    }

    #[test]
    fn conjugate_inverts_unit() {
        let q = Quat::new(0.3, 0.1, 0.2, 0.9).normalize();
        let prod = q.hamilton(q.conjugate());
        assert!((prod.x).abs() < 1e-3);
        assert!((prod.y).abs() < 1e-3);
        assert!((prod.z).abs() < 1e-3);
        assert!((prod.w - 1.0).abs() < 1e-3);
    }

    #[test]
    fn axis_angle_90deg_about_z() {
        let q = Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), core::f32::consts::FRAC_PI_2);
        let v = Vec3::new(1.0, 0.0, 0.0);
        let r = q.rotate_vec3(v);
        assert!((r.x).abs() < 1e-2);
        assert!((r.y - 1.0).abs() < 1e-2);
        assert!((r.z).abs() < 1e-2);
    }

    #[test]
    fn slerp_endpoints() {
        let a = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0);
        let b = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.0);
        let r0 = slerp(a, b, 0.0);
        let r1 = slerp(a, b, 1.0);
        assert!((r0.w - a.w).abs() < 1e-3);
        assert!((r1.w - b.w).abs() < 1e-3);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// A unit quaternion times its conjugate is (approximately) the identity.
        #[test]
        fn unit_quat_conjugate_is_identity(
            x in -1.0f32..1.0, y in -1.0f32..1.0, z in -1.0f32..1.0, w in -1.0f32..1.0) {
            let q = Quat::new(x, y, z, w).normalize();
        let prod = q.hamilton(q.conjugate());
            prop_assert!((prod.x).abs() < 1e-2, "x={}", prod.x);
            prop_assert!((prod.y).abs() < 1e-2, "y={}", prod.y);
            prop_assert!((prod.z).abs() < 1e-2, "z={}", prod.z);
            prop_assert!((prod.w - 1.0).abs() < 1e-2, "w={}", prod.w);
        }

        /// Rotating by the identity quaternion leaves a vector unchanged.
        #[test]
        fn identity_rotation_is_noop(x in -10.0f32..10.0, y in -10.0f32..10.0, z in -10.0f32..10.0) {
            let v = Vec3::new(x, y, z);
            let r = Quat::identity().rotate_vec3(v);
            prop_assert!((r.x - v.x).abs() < 1e-2);
            prop_assert!((r.y - v.y).abs() < 1e-2);
            prop_assert!((r.z - v.z).abs() < 1e-2);
        }

        /// A rotation preserves vector length (within the fast-math tolerance).
        #[test]
        fn rotation_preserves_length(
            ax in -1.0f32..1.0, ay in -1.0f32..1.0, az in -1.0f32..1.0,
            ang in -core::f32::consts::PI..core::f32::consts::PI,
            x in -10.0f32..10.0, y in -10.0f32..10.0, z in -10.0f32..10.0) {
            let q = Quat::from_axis_angle(Vec3::new(ax, ay, az), ang);
            let v = Vec3::new(x, y, z);
            let r = q.rotate_vec3(v);
            prop_assert!((r.length() - v.length()).abs() < 1e-2);
        }
    }
}
