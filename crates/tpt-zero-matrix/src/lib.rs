#![no_std]
//! `tpt-zero-matrix`: minimal `f32` matrix types for `#![no_std]`.
//!
//! Provides [`Mat3`] and [`Mat4`], stored **column-major** as arrays of
//! [`Vec3`] / [`Vec4`] columns. Operations include multiplication, transpose,
//! determinant, inverse, and the common view/projection builders
//! (`look_at`, `perspective`, `orthographic`). Length/normalize use
//! [`tpt-zero-fast-math`](https://docs.rs/tpt-zero-fast-math).
//!
//! # Scope (v0.1)
//!
//! `f32`-only. No `Quat` conversion in v0.1 (see `tpt-zero-quat`).

use tpt_zero_fast_math::tan;
use tpt_zero_vec::{Vec3, Vec4};

/// A 3x3 matrix stored column-major (`columns[c][r]`).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat3 {
    pub columns: [Vec3; 3],
}

/// A 4x4 matrix stored column-major (`columns[c][r]`).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat4 {
    pub columns: [Vec4; 4],
}

impl Mat3 {
    /// The identity matrix.
    #[inline]
    pub fn identity() -> Self {
        Mat3 {
            columns: [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
        }
    }

    /// Construct from three column vectors.
    #[inline]
    pub fn from_columns(c0: Vec3, c1: Vec3, c2: Vec3) -> Self {
        Mat3 {
            columns: [c0, c1, c2],
        }
    }

    /// Element `(row, col)`.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.columns[col][row]
    }

    /// Transpose (rows become columns).
    #[inline]
    pub fn transpose(&self) -> Self {
        Mat3 {
            columns: [
                Vec3::new(self.get(0, 0), self.get(1, 0), self.get(2, 0)),
                Vec3::new(self.get(0, 1), self.get(1, 1), self.get(2, 1)),
                Vec3::new(self.get(0, 2), self.get(1, 2), self.get(2, 2)),
            ],
        }
    }

    /// Determinant.
    #[inline]
    pub fn determinant(&self) -> f32 {
        let m = &self.columns;
        let a = m[0].x;
        let b = m[1].x;
        let c = m[2].x;
        let d = m[0].y;
        let e = m[1].y;
        let f = m[2].y;
        let g = m[0].z;
        let h = m[1].z;
        let i = m[2].z;
        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    /// Inverse. Returns `None` if the matrix is singular (determinant ~ 0).
    #[inline]
    pub fn invert(&self) -> Option<Mat3> {
        let a = self.get(0, 0);
        let b = self.get(0, 1);
        let c = self.get(0, 2);
        let d = self.get(1, 0);
        let e = self.get(1, 1);
        let f = self.get(1, 2);
        let g = self.get(2, 0);
        let h = self.get(2, 1);
        let i = self.get(2, 2);
        let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;
        // Inverse is (1/det) * adjugate; adjugate is the transpose of the
        // cofactor matrix. Stored column-major: column k is the k-th column of
        // the inverse, i.e. (C[0][k], C[1][k], C[2][k]) / det where C is the
        // cofactor matrix.
        let c0 = Vec3::new(e * i - f * h, f * g - d * i, d * h - e * g) * inv_det;
        let c1 = Vec3::new(c * h - b * i, a * i - c * g, b * g - a * h) * inv_det;
        let c2 = Vec3::new(b * f - c * e, c * d - a * f, a * e - b * d) * inv_det;
        Some(Mat3::from_columns(c0, c1, c2))
    }

    /// Multiply two matrices (`self * rhs`).
    #[inline]
    pub fn mul(&self, rhs: &Mat3) -> Mat3 {
        Mat3 {
            columns: [
                self.transform(rhs.columns[0]),
                self.transform(rhs.columns[1]),
                self.transform(rhs.columns[2]),
            ],
        }
    }

    /// Transform a `Vec3` by this matrix (as a column vector `M * v`).
    #[inline]
    pub fn transform(&self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.columns[0].x * v.x + self.columns[1].x * v.y + self.columns[2].x * v.z,
            self.columns[0].y * v.x + self.columns[1].y * v.y + self.columns[2].y * v.z,
            self.columns[0].z * v.x + self.columns[1].z * v.y + self.columns[2].z * v.z,
        )
    }
}

impl Mat4 {
    /// The identity matrix.
    #[inline]
    pub fn identity() -> Self {
        Mat4 {
            columns: [
                Vec4::new(1.0, 0.0, 0.0, 0.0),
                Vec4::new(0.0, 1.0, 0.0, 0.0),
                Vec4::new(0.0, 0.0, 1.0, 0.0),
                Vec4::new(0.0, 0.0, 0.0, 1.0),
            ],
        }
    }

    /// Construct from four column vectors.
    #[inline]
    pub fn from_columns(c0: Vec4, c1: Vec4, c2: Vec4, c3: Vec4) -> Self {
        Mat4 {
            columns: [c0, c1, c2, c3],
        }
    }

    /// Construct from a 4x4 row-major nested array (converted to column-major).
    #[inline]
    pub fn from_row_major(m: [[f32; 4]; 4]) -> Self {
        Mat4 {
            columns: [
                Vec4::new(m[0][0], m[1][0], m[2][0], m[3][0]),
                Vec4::new(m[0][1], m[1][1], m[2][1], m[3][1]),
                Vec4::new(m[0][2], m[1][2], m[2][2], m[3][2]),
                Vec4::new(m[0][3], m[1][3], m[2][3], m[3][3]),
            ],
        }
    }

    /// Element `(row, col)`.
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.columns[col][row]
    }

    /// Transpose.
    #[inline]
    pub fn transpose(&self) -> Self {
        Mat4 {
            columns: [
                Vec4::new(
                    self.get(0, 0),
                    self.get(1, 0),
                    self.get(2, 0),
                    self.get(3, 0),
                ),
                Vec4::new(
                    self.get(0, 1),
                    self.get(1, 1),
                    self.get(2, 1),
                    self.get(3, 1),
                ),
                Vec4::new(
                    self.get(0, 2),
                    self.get(1, 2),
                    self.get(2, 2),
                    self.get(3, 2),
                ),
                Vec4::new(
                    self.get(0, 3),
                    self.get(1, 3),
                    self.get(2, 3),
                    self.get(3, 3),
                ),
            ],
        }
    }

    /// Determinant.
    #[inline]
    pub fn determinant(&self) -> f32 {
        let m = &self.columns;
        let a = m[0].x;
        let b = m[1].x;
        let c = m[2].x;
        let d = m[3].x;
        let e = m[0].y;
        let f = m[1].y;
        let g = m[2].y;
        let h = m[3].y;
        let i = m[0].z;
        let j = m[1].z;
        let k = m[2].z;
        let l = m[3].z;
        let mm = m[0].w;
        let n = m[1].w;
        let o = m[2].w;
        let p = m[3].w;
        a * (f * (k * p - l * o) - g * (j * p - l * n) + h * (j * o - k * n))
            - b * (e * (k * p - l * o) - g * (i * p - l * mm) + h * (i * o - k * mm))
            + c * (e * (j * p - l * n) - f * (i * p - l * mm) + h * (i * n - j * mm))
            - d * (e * (j * o - k * n) - f * (i * o - k * mm) + g * (i * n - j * mm))
    }

    /// Inverse via cofactor expansion. Returns `None` if near-singular.
    #[inline]
    pub fn invert(&self) -> Option<Mat4> {
        let det = self.determinant();
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;
        let m = &self.columns;
        let a = m[0].x;
        let b = m[1].x;
        let c = m[2].x;
        let d = m[3].x;
        let e = m[0].y;
        let f = m[1].y;
        let g = m[2].y;
        let h = m[3].y;
        let i = m[0].z;
        let j = m[1].z;
        let k = m[2].z;
        let l = m[3].z;
        let mm = m[0].w;
        let n = m[1].w;
        let o = m[2].w;
        let p = m[3].w;

        let c0 = Vec4::new(
            f * (k * p - l * o) - g * (j * p - l * n) + h * (j * o - k * n),
            -(e * (k * p - l * o) - g * (i * p - l * mm) + h * (i * o - k * mm)),
            e * (j * p - l * n) - f * (i * p - l * mm) + h * (i * n - j * mm),
            -(e * (j * o - k * n) - f * (i * o - k * mm) + g * (i * n - j * mm)),
        ) * inv_det;
        let c1 = Vec4::new(
            -(b * (k * p - l * o) - c * (j * p - l * n) + d * (j * o - k * n)),
            a * (k * p - l * o) - c * (i * p - l * mm) + d * (i * o - k * mm),
            -(a * (j * p - l * n) - b * (i * p - l * mm) + d * (i * n - j * mm)),
            a * (j * o - k * n) - b * (i * o - k * mm) + c * (i * n - j * mm),
        ) * inv_det;
        let c2 = Vec4::new(
            b * (g * p - h * o) - c * (f * p - h * n) + d * (f * o - g * n),
            -(a * (g * p - h * o) - c * (e * p - h * mm) + d * (e * o - g * mm)),
            a * (f * p - h * n) - b * (e * p - h * mm) + d * (e * n - f * mm),
            -(a * (f * o - g * n) - b * (e * o - g * mm) + c * (e * n - f * mm)),
        ) * inv_det;
        let c3 = Vec4::new(
            -(b * (g * l - h * k) - c * (f * l - h * j) + d * (f * k - g * j)),
            a * (g * l - h * k) - c * (e * l - h * i) + d * (e * k - g * i),
            -(a * (f * l - h * j) - b * (e * l - h * i) + d * (e * j - f * i)),
            a * (f * k - g * j) - b * (e * k - g * i) + c * (e * j - f * i),
        ) * inv_det;
        Some(Mat4::from_columns(c0, c1, c2, c3))
    }

    /// Multiply two matrices (`self * rhs`).
    #[inline]
    pub fn mul(&self, rhs: &Mat4) -> Mat4 {
        Mat4 {
            columns: [
                self.transform(rhs.columns[0]),
                self.transform(rhs.columns[1]),
                self.transform(rhs.columns[2]),
                self.transform(rhs.columns[3]),
            ],
        }
    }

    /// Transform a `Vec4` by this matrix (`M * v`).
    #[inline]
    pub fn transform(&self, v: Vec4) -> Vec4 {
        Vec4::new(
            self.columns[0].x * v.x
                + self.columns[1].x * v.y
                + self.columns[2].x * v.z
                + self.columns[3].x * v.w,
            self.columns[0].y * v.x
                + self.columns[1].y * v.y
                + self.columns[2].y * v.z
                + self.columns[3].y * v.w,
            self.columns[0].z * v.x
                + self.columns[1].z * v.y
                + self.columns[2].z * v.z
                + self.columns[3].z * v.w,
            self.columns[0].w * v.x
                + self.columns[1].w * v.y
                + self.columns[2].w * v.z
                + self.columns[3].w * v.w,
        )
    }

    /// Build a right-handed look-at view matrix. `up` need not be normalized.
    #[inline]
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
        let f = (center - eye).normalize();
        let s = f.cross(up.normalize());
        let u = s.cross(f);
        Mat4::from_columns(
            Vec4::new(s.x, u.x, -f.x, 0.0),
            Vec4::new(s.y, u.y, -f.y, 0.0),
            Vec4::new(s.z, u.z, -f.z, 0.0),
            Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        )
    }

    /// Build a right-handed perspective projection matrix.
    #[inline]
    pub fn perspective(fovy_rad: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / tan(fovy_rad / 2.0);
        let nf = 1.0 / (near - far);
        Mat4::from_columns(
            Vec4::new(f / aspect, 0.0, 0.0, 0.0),
            Vec4::new(0.0, f, 0.0, 0.0),
            Vec4::new(0.0, 0.0, (far + near) * nf, -1.0),
            Vec4::new(0.0, 0.0, 2.0 * far * near * nf, 0.0),
        )
    }

    /// Build a right-handed orthographic projection matrix.
    #[inline]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
        let lr = 1.0 / (left - right);
        let bt = 1.0 / (bottom - top);
        let nf = 1.0 / (near - far);
        Mat4::from_columns(
            Vec4::new(-2.0 * lr, 0.0, 0.0, 0.0),
            Vec4::new(0.0, -2.0 * bt, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 2.0 * nf, 0.0),
            Vec4::new(
                (left + right) * lr,
                (top + bottom) * bt,
                (far + near) * nf,
                1.0,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mat3_identity() {
        let m = Mat3::identity();
        for r in 0..3 {
            for c in 0..3 {
                let want = if r == c { 1.0 } else { 0.0 };
                assert!((m.get(r, c) - want).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn mat3_transpose() {
        let m = Mat3::from_columns(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let t = m.transpose();
        assert!((t.get(0, 1) - 4.0).abs() < 1e-5);
        assert!((t.get(2, 0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn mat3_determinant() {
        let m = Mat3::from_columns(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 3.0),
        );
        assert!((m.determinant() - 6.0).abs() < 1e-5);
    }

    #[test]
    fn mat3_inverse() {
        let m = Mat3::from_columns(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 3.0),
        );
        let inv = m.invert().unwrap();
        let prod = m.mul(&inv);
        for r in 0..3 {
            for c in 0..3 {
                let want = if r == c { 1.0 } else { 0.0 };
                assert!(
                    (prod.get(r, c) - want).abs() < 1e-4,
                    "({},{})={}",
                    r,
                    c,
                    prod.get(r, c)
                );
            }
        }
    }

    #[test]
    fn mat3_inverse_singular() {
        let m = Mat3::from_columns(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(2.0, 4.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        assert!(m.invert().is_none());
    }

    #[test]
    fn mat4_identity_mul() {
        let m = Mat4::from_columns(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 1.0, 2.0, 3.0),
            Vec4::new(4.0, 5.0, 6.0, 7.0),
        );
        let prod = m.mul(&Mat4::identity());
        for r in 0..4 {
            for c in 0..4 {
                assert!((prod.get(r, c) - m.get(r, c)).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn mat4_determinant_identity() {
        assert!((Mat4::identity().determinant() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn mat4_inverse() {
        let m = Mat4::from_columns(
            Vec4::new(2.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 3.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 4.0, 0.0),
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );
        let inv = m.invert().unwrap();
        let prod = m.mul(&inv);
        for r in 0..4 {
            for c in 0..4 {
                let want = if r == c { 1.0 } else { 0.0 };
                assert!(
                    (prod.get(r, c) - want).abs() < 1e-4,
                    "({},{})={}",
                    r,
                    c,
                    prod.get(r, c)
                );
            }
        }
    }

    #[test]
    fn mat4_transform() {
        let m = Mat4::identity();
        let v = Vec4::new(1.0, 2.0, 3.0, 1.0);
        assert_eq!(m.transform(v), v);
    }

    #[test]
    fn look_at_basics() {
        let view = Mat4::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        // The origin lies in front of the eye along -z, so its view-space z is
        // negative (right-handed view space looks down -z).
        let p = view.transform(Vec4::new(0.0, 0.0, 0.0, 1.0));
        assert!((p.z + 5.0).abs() < 1e-2, "p.z={}", p.z);
    }

    #[test]
    fn perspective_basics() {
        let p = Mat4::perspective(1.0, 1.0, 0.1, 100.0);
        // A point on the near plane, centered, should map to NDC z = -1.
        let v = p.transform(Vec4::new(0.0, 0.0, -0.1, 1.0));
        let ndc_z = v.z / v.w;
        assert!((ndc_z + 1.0).abs() < 1e-3, "ndc_z={}", ndc_z);
    }

    #[test]
    fn from_row_major_roundtrip() {
        let m = Mat4::from_row_major([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0, 7.0],
        ]);
        assert!((m.get(0, 0) - 1.0).abs() < 1e-5);
        assert!((m.get(2, 1) - 1.0).abs() < 1e-5);
        assert!((m.get(3, 3) - 7.0).abs() < 1e-5);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// `M * M^-1` is approximately the identity for random non-singular
        /// matrices (within the fast-math tolerance).
        #[test]
        fn mat3_inverse_identity(
            a in -3.0f32..3.0, b in -3.0f32..3.0, c in -3.0f32..3.0,
            d in -3.0f32..3.0, e in -3.0f32..3.0, f in -3.0f32..3.0,
            g in -3.0f32..3.0, h in -3.0f32..3.0, i in -3.0f32..3.0) {
            let m = Mat3::from_columns(
                Vec3::new(a, b, c), Vec3::new(d, e, f), Vec3::new(g, h, i));
            if m.determinant().abs() > 0.05 {
                let inv = m.invert().unwrap();
                let prod = m.mul(&inv);
                for r in 0..3 {
                    for c2 in 0..3 {
                        let want = if r == c2 { 1.0 } else { 0.0 };
                        prop_assert!((prod.get(r, c2) - want).abs() < 1e-2,
                            "({},{})={}", r, c2, prod.get(r, c2));
                    }
                }
            }
        }

        /// Matrix multiplication is associative enough: `(A*B)*C == A*(B*C)`
        /// within tolerance.
        #[test]
        fn mat4_mul_associative(
            a in -2.0f32..2.0, b in -2.0f32..2.0, c in -2.0f32..2.0, d in -2.0f32..2.0,
            e in -2.0f32..2.0, f in -2.0f32..2.0, g in -2.0f32..2.0, h in -2.0f32..2.0,
            i in -2.0f32..2.0, j in -2.0f32..2.0, k in -2.0f32..2.0, l in -2.0f32..2.0) {
            let x = Mat4::from_columns(
                Vec4::new(a, b, c, d), Vec4::new(e, f, g, h),
                Vec4::new(i, j, k, l), Vec4::new(a+b, c+d, e+f, g+h));
            let y = x; // use the same matrix to keep it simple
            let z = x;
            let left = x.mul(&y).mul(&z);
            let right = x.mul(&y.mul(&z));
            for r in 0..4 {
                for c2 in 0..4 {
                    prop_assert!((left.get(r, c2) - right.get(r, c2)).abs() < 1e-2);
                }
            }
        }
    }
}
