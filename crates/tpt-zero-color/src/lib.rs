#![no_std]
//! `tpt-zero-color`: small color types for `#![no_std]`.
//!
//! Provides:
//!
//! - [`Rgb`] / [`Rgba`] — 8-bit-per-channel red/green/blue (and alpha).
//! - [`Hsv`] — hue/saturation/value, also 8-bit channels.
//! - Conversions [`Rgb`] <-> [`Hsv`] (and `Rgba` -> `Hsv`, dropping alpha).
//! - `#RRGGBB` and `#RRGGBBAA` hex parsing ([`Rgb::from_hex`] /
//!   [`Rgba::from_hex`]) and formatting ([`Rgb::to_hex`] / [`Rgba::to_hex`]).

/// An RGB color with 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

/// An RGBA color with 8-bit channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgba {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

/// An HSV color with 8-bit channels; `h` is degrees in `[0, 360)`, `s`/`v` in
/// `[0, 255]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hsv {
    /// Hue, in degrees, `[0, 360)`.
    pub h: u16,
    /// Saturation, `[0, 255]`.
    pub s: u8,
    /// Value, `[0, 255]`.
    pub v: u8,
}

impl Rgb {
    /// Construct a new `Rgb`.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    /// Parse `#RRGGBB` (leading `#` optional).
    ///
    /// Returns `None` if the slice is not exactly 6 hex digits.
    pub fn from_hex(s: &[u8]) -> Option<Self> {
        let s = strip_hash(s);
        if s.len() != 6 {
            return None;
        }
        let r = hex2(s.get(0..2)?)?;
        let g = hex2(s.get(2..4)?)?;
        let b = hex2(s.get(4..6)?)?;
        Some(Rgb { r, g, b })
    }

    /// Write `#RRGGBB` (with leading `#`) into `buf`. Returns the written slice.
    pub fn to_hex<'a>(&self, buf: &'a mut [u8]) -> Option<&'a mut [u8]> {
        if buf.len() < 7 {
            return None;
        }
        buf[0] = b'#';
        put_hex(&mut buf[1..3], self.r);
        put_hex(&mut buf[3..5], self.g);
        put_hex(&mut buf[5..7], self.b);
        Some(&mut buf[..7])
    }

    /// Convert to HSV.
    pub fn to_hsv(&self) -> Hsv {
        rgb_to_hsv(self.r, self.g, self.b)
    }
}

impl Rgba {
    /// Construct a new `Rgba`.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Rgba { r, g, b, a }
    }

    /// Parse `#RRGGBB` or `#RRGGBBAA` (leading `#` optional).
    ///
    /// Returns `None` if the slice is not 6 or 8 hex digits.
    pub fn from_hex(s: &[u8]) -> Option<Self> {
        let s = strip_hash(s);
        if s.len() == 6 {
            let r = hex2(s.get(0..2)?)?;
            let g = hex2(s.get(2..4)?)?;
            let b = hex2(s.get(4..6)?)?;
            Some(Rgba { r, g, b, a: 255 })
        } else if s.len() == 8 {
            let r = hex2(s.get(0..2)?)?;
            let g = hex2(s.get(2..4)?)?;
            let b = hex2(s.get(4..6)?)?;
            let a = hex2(s.get(6..8)?)?;
            Some(Rgba { r, g, b, a })
        } else {
            None
        }
    }

    /// Write `#RRGGBBAA` (with leading `#`) into `buf`. Returns the written
    /// slice.
    pub fn to_hex<'a>(&self, buf: &'a mut [u8]) -> Option<&'a mut [u8]> {
        if buf.len() < 9 {
            return None;
        }
        buf[0] = b'#';
        put_hex(&mut buf[1..3], self.r);
        put_hex(&mut buf[3..5], self.g);
        put_hex(&mut buf[5..7], self.b);
        put_hex(&mut buf[7..9], self.a);
        Some(&mut buf[..9])
    }

    /// Drop the alpha, returning the RGB triple.
    pub const fn rgb(&self) -> Rgb {
        Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

fn strip_hash(s: &[u8]) -> &[u8] {
    if s.first() == Some(&b'#') {
        &s[1..]
    } else {
        s
    }
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn hex2(s: &[u8]) -> Option<u8> {
    Some(hex_val(s[0])? * 16 + hex_val(s[1])?)
}

fn put_hex(out: &mut [u8], v: u8) {
    const D: &[u8; 16] = b"0123456789abcdef";
    out[0] = D[(v >> 4) as usize];
    out[1] = D[(v & 0xF) as usize];
}

fn rgb_to_hsv(r: u8, g: u8, b: u8) -> Hsv {
    let r = r as u32;
    let g = g as u32;
    let b = b as u32;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0 {
        0i32
    } else if max == r {
        (((g as i32 - b as i32) * 60) / d as i32 + 360) % 360
        } else if max == g {
            120 + ((b as i32 - r as i32) * 60) / d as i32
        } else {
            240 + ((r as i32 - g as i32) * 60) / d as i32
        };
    let s = if max == 0 { 0u8 } else { ((d * 255) / max) as u8 };
    Hsv {
        h: h as u16,
        s,
        v: max as u8,
    }
}

impl Hsv {
    /// Convert to [`Rgb`].
    pub fn to_rgb(&self) -> Rgb {
        if self.s == 0 {
            return Rgb::new(self.v, self.v, self.v);
        }
        let region = (self.h / 60) as u32;
        let remainder = (self.h % 60) as u32;
        let v = self.v as u32;
        let s = self.s as u32;
        let p = (v * (255 - s)) / 255;
        let q = (v * (255 - (s * remainder) / 60)) / 255;
        let t = (v * (255 - (s * (60 - remainder)) / 60)) / 255;
        let (r, g, b) = match region {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, t),
        };
        Rgb {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let c = Rgb::new(0x12, 0x34, 0xAB);
        let mut buf = [0u8; 7];
        let s = c.to_hex(&mut buf).unwrap();
        assert_eq!(s, b"#1234ab");
        assert_eq!(Rgb::from_hex(b"#1234ab"), Some(c));
        assert_eq!(Rgb::from_hex(b"1234ab"), Some(c));
    }

    #[test]
    fn rgba_hex() {
        let c = Rgba::new(0xFF, 0x00, 0x80, 0x40);
        let mut buf = [0u8; 9];
        let s = c.to_hex(&mut buf).unwrap();
        assert_eq!(s, b"#ff008040");
        assert_eq!(Rgba::from_hex(b"#ff008040"), Some(c));
        assert_eq!(Rgba::from_hex(b"ff0080"), Some(Rgba::new(0xFF, 0x00, 0x80, 255)));
    }

    #[test]
    fn hsv_roundtrip_gray() {
        let c = Rgb::new(128, 128, 128);
        let hsv = c.to_hsv();
        assert_eq!(hsv.to_rgb(), c);
    }

    #[test]
    fn hsv_black_white() {
        assert_eq!(Rgb::new(0, 0, 0).to_hsv(), Hsv { h: 0, s: 0, v: 0 });
        assert_eq!(Rgb::new(255, 255, 255).to_hsv(), Hsv { h: 0, s: 0, v: 255 });
    }
}

impl Rgba {
    /// Construct an `Rgba` from an `Rgb` and an alpha.
    pub const fn with_alpha(self, a: u8) -> Self {
        Rgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// RGB -> HSV -> RGB is within one 8-bit step per channel.
        #[test]
        fn rgb_hsv_rgb(r in any::<u8>(), g in any::<u8>(), b in any::<u8>()) {
            let c = Rgb::new(r, g, b);
            let back = c.to_hsv().to_rgb();
            prop_assert!(
                (back.r as i16 - r as i16).abs() <= 1
                    && (back.g as i16 - g as i16).abs() <= 1
                    && (back.b as i16 - b as i16).abs() <= 1,
                "rgb {:?} -> hsv -> rgb {:?}",
                c,
                back
            );
        }

        /// Hex parse/format round-trips for RGB.
        #[test]
        fn rgb_hex_roundtrip(r in any::<u8>(), g in any::<u8>(), b in any::<u8>()) {
            let c = Rgb::new(r, g, b);
            let mut buf = [0u8; 7];
            let s = c.to_hex(&mut buf).unwrap();
            prop_assert_eq!(Rgb::from_hex(s), Some(c));
        }
    }
}
