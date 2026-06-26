//! Hardware-independent LED wire encoding.
//!
//! Turns the rendered `[Pixel]` strand into the byte stream a transmitter
//! sends. Two families, selected by `[ledinfo].type`:
//!
//! - **WS281x** (clockless: WS2811/2812/2813/2815, SK6812): 3 bytes per pixel in
//!   a configurable color order. The firmware's RMT encoder turns each byte's
//!   bits into pulses.
//! - **APA102** (clocked, SPI): a start frame, a 4-byte frame per pixel
//!   (`0xE0|brightness`, then B, G, R — matching the Go `wledapa` path), and an
//!   end frame. The firmware's SPI driver clocks these bytes out.
//!
//! Lives in `slisko-core` (not the firmware) so it is `no_std` and host-tested.

use alloc::vec::Vec;

use crate::pixel::Pixel;

/// LED chip family. Parsed from the config `[ledinfo].type` string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedType {
    Ws2811,
    Ws2812,
    Ws2813,
    Ws2815,
    Sk6812,
    Apa102,
}

impl LedType {
    /// Parse a config type string (case-insensitive). Returns `None` if unknown.
    pub fn parse(s: &str) -> Option<LedType> {
        // ASCII-uppercase without allocating.
        let mut buf = [0u8; 16];
        let bytes = s.as_bytes();
        if bytes.len() > buf.len() {
            return None;
        }
        for (i, b) in bytes.iter().enumerate() {
            buf[i] = b.to_ascii_uppercase();
        }
        match &buf[..bytes.len()] {
            b"WS2811" => Some(LedType::Ws2811),
            b"WS2812" | b"WS2812B" => Some(LedType::Ws2812),
            b"WS2813" => Some(LedType::Ws2813),
            b"WS2815" => Some(LedType::Ws2815),
            b"SK6812" => Some(LedType::Sk6812),
            b"APA102" => Some(LedType::Apa102),
            _ => None,
        }
    }

    /// True for single-wire clockless chips (driven over RMT); false for APA102
    /// (clock + data, driven over SPI).
    pub fn is_clockless(self) -> bool {
        !matches!(self, LedType::Apa102)
    }

    /// The wire color order this chip expects by default.
    pub fn default_color_order(self) -> ColorOrder {
        match self {
            // WS2811 strings are commonly wired RGB; the rest of the WS/SK
            // family is GRB. APA102 order is handled by its own framing.
            LedType::Ws2811 => ColorOrder::Rgb,
            _ => ColorOrder::Grb,
        }
    }
}

/// Byte order on the wire for clockless chips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorOrder {
    Rgb,
    Rbg,
    Grb,
    Gbr,
    Brg,
    Bgr,
}

impl ColorOrder {
    /// Reorder an `[r, g, b]` triple into this wire order.
    #[inline]
    pub fn apply(self, r: u8, g: u8, b: u8) -> [u8; 3] {
        match self {
            ColorOrder::Rgb => [r, g, b],
            ColorOrder::Rbg => [r, b, g],
            ColorOrder::Grb => [g, r, b],
            ColorOrder::Gbr => [g, b, r],
            ColorOrder::Brg => [b, r, g],
            ColorOrder::Bgr => [b, g, r],
        }
    }
}

/// Encode a pixel slice as WS281x bytes (3 per pixel) in `order`, appending to
/// `out`. `out` is cleared first.
pub fn encode_ws281x(leds: &[Pixel], order: ColorOrder, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(leds.len() * 3);
    for p in leds {
        let [r, g, b] = p.to_rgb8();
        out.extend_from_slice(&order.apply(r, g, b));
    }
}

/// APA102 5-bit global brightness range (`0..=31`).
pub const APA102_MAX_BRIGHTNESS: u8 = 31;

/// Encode a pixel slice as a full APA102 SPI frame into `out` (cleared first):
/// 4-byte start frame, one 4-byte frame per pixel (`0xE0|brightness`, B, G, R),
/// then an end frame of `ceil(n/16)` `0xFF` bytes to clock the last pixel out.
pub fn encode_apa102(leds: &[Pixel], brightness: u8, out: &mut Vec<u8>) {
    let b5 = brightness.min(APA102_MAX_BRIGHTNESS);
    let header = 0xE0 | b5;
    let n = leds.len();
    let end_len = n.div_ceil(16).max(4);

    out.clear();
    out.reserve(4 + n * 4 + end_len);

    // Start frame.
    out.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    // Per-pixel frames (brightness, B, G, R).
    for p in leds {
        let [r, g, b] = p.to_rgb8();
        out.extend_from_slice(&[header, b, g, r]);
    }
    // End frame.
    for _ in 0..end_len {
        out.push(0xFF);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel::Pixel;

    fn px(r: f32, g: f32, b: f32) -> Pixel {
        let mut p = Pixel::new();
        p.set_color(r, g, b);
        p
    }

    #[test]
    fn parse_types() {
        assert_eq!(LedType::parse("WS2815"), Some(LedType::Ws2815));
        assert_eq!(LedType::parse("ws2812b"), Some(LedType::Ws2812));
        assert_eq!(LedType::parse("APA102"), Some(LedType::Apa102));
        assert_eq!(LedType::parse("nonsense"), None);
        assert!(LedType::Ws2815.is_clockless());
        assert!(!LedType::Apa102.is_clockless());
    }

    #[test]
    fn ws281x_grb_order() {
        let leds = [px(1.0, 0.5, 0.0)]; // r=255, g=127, b=0
        let mut out = Vec::new();
        encode_ws281x(&leds, ColorOrder::Grb, &mut out);
        assert_eq!(out, alloc::vec![127, 255, 0]); // G, R, B
    }

    #[test]
    fn apa102_frame_layout() {
        let leds = [px(1.0, 0.0, 0.0), px(0.0, 1.0, 0.0)];
        let mut out = Vec::new();
        encode_apa102(&leds, 31, &mut out);
        // start(4) + 2*4 + end(max(ceil(2/16),4)=4) = 16
        assert_eq!(out.len(), 16);
        assert_eq!(&out[0..4], &[0, 0, 0, 0]); // start
        // pixel 0: red -> header, B=0, G=0, R=255
        assert_eq!(&out[4..8], &[0xFF, 0, 0, 255]);
        // pixel 1: green -> header, B=0, G=255, R=0
        assert_eq!(&out[8..12], &[0xFF, 0, 255, 0]);
        assert_eq!(&out[12..16], &[0xFF, 0xFF, 0xFF, 0xFF]); // end
    }

    #[test]
    fn apa102_clamps_brightness() {
        let leds = [px(1.0, 1.0, 1.0)];
        let mut out = Vec::new();
        encode_apa102(&leds, 200, &mut out); // clamped to 31
        assert_eq!(out[4], 0xE0 | 31);
    }
}
