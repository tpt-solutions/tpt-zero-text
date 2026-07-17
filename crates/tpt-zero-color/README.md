# tpt-zero-color

Small color types for `#![no_std]`, with **zero** external dependencies.

- [`Rgb`] / [`Rgba`] — 8-bit red/green/blue (+ alpha).
- [`Hsv`] — hue/saturation/value, also 8-bit channels.
- [`Rgb`] <-> [`Hsv`] conversion.
- `#RRGGBB` / `#RRGGBBAA` hex parsing ([`Rgb::from_hex`] / [`Rgba::from_hex`])
  and formatting ([`Rgb::to_hex`] / [`Rgba::to_hex`]).

```rust
use tpt_zero_color::Rgb;

let c = Rgb::from_hex(b"#1e90ff").unwrap();
let mut buf = [0u8; 7];
assert_eq!(c.to_hex(&mut buf).unwrap(), b"#1e90ff");
let hsv = c.to_hsv();
```

## `no_std`

`#![no_std]` with **zero** external dependencies.

## HSV round-trip

`Rgb` <-> `Hsv` conversion is **lossy** for saturated colors: the 8-bit
`h`/`s`/`v` fields cannot represent every one of the 16.7M possible `Rgb`
values. Achromatic (grayscale, `s == 0`) colors round-trip exactly; other
colors are approximate.

## License

Licensed under MIT or Apache-2.0 at your option.
