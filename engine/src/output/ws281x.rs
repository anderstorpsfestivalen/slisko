use alloc::vec::Vec;

use crate::pixel::{Pixel, clamp01};

/// Supported clockless WS281x/SK6812 LED chips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ws281xType {
    Ws2811,
    Ws2812,
    Ws2813,
    Ws2815,
    Sk6812,
}

impl Ws281xType {
    /// Parse a config type string case-insensitively.
    pub fn parse(s: &str) -> Option<Self> {
        let mut buf = [0u8; 16];
        let bytes = s.as_bytes();
        if bytes.len() > buf.len() {
            return None;
        }
        for (i, byte) in bytes.iter().enumerate() {
            buf[i] = byte.to_ascii_uppercase();
        }
        match &buf[..bytes.len()] {
            b"WS2811" => Some(Self::Ws2811),
            b"WS2812" | b"WS2812B" => Some(Self::Ws2812),
            b"WS2813" => Some(Self::Ws2813),
            b"WS2815" => Some(Self::Ws2815),
            b"SK6812" => Some(Self::Sk6812),
            _ => None,
        }
    }

    pub fn default_color_order(self) -> ColorOrder {
        match self {
            Self::Ws2811 => ColorOrder::Rgb,
            Self::Ws2812 | Self::Ws2813 | Self::Ws2815 | Self::Sk6812 => ColorOrder::Grb,
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
    #[inline]
    pub fn apply(self, r: u8, g: u8, b: u8) -> [u8; 3] {
        match self {
            Self::Rgb => [r, g, b],
            Self::Rbg => [r, b, g],
            Self::Grb => [g, r, b],
            Self::Gbr => [g, b, r],
            Self::Brg => [b, r, g],
            Self::Bgr => [b, g, r],
        }
    }
}

/// Smallest positive `f32` bit pattern that rounds to each PWM value 1..=255
/// under the IEC 61966-2-1 EOTF. Positive finite `f32` bit patterns sort in
/// numeric order, so the output conversion is an eight-step integer search.
///
/// This costs 1020 bytes of flash and avoids running `powf` three times per LED
/// on every frame. The transition points were exhaustively derived from the
/// reference `f32` transfer function below and are checked at every boundary
/// in the tests.
const SRGB_TO_PWM_TRANSITIONS: [u32; 255] = [
    0x3ccf87d9, 0x3d8d9754, 0x3dc9e484, 0x3df93ad7, 0x3e1096c4, 0x3e221c1a, 0x3e31dafc, 0x3e403e2a,
    0x3e4d8cb8, 0x3e59f8ae, 0x3e65a6ce, 0x3e70b30a, 0x3e7b332c, 0x3e829c49, 0x3e8768ad, 0x3e8c0494,
    0x3e9074de, 0x3e94bda7, 0x3e98e275, 0x3e9ce650, 0x3ea0cbdd, 0x3ea49569, 0x3ea844fe, 0x3eabdc6c,
    0x3eaf5d4c, 0x3eb2c910, 0x3eb62103, 0x3eb96653, 0x3ebc9a0e, 0x3ebfbd2d, 0x3ec2d091, 0x3ec5d509,
    0x3ec8cb55, 0x3ecbb422, 0x3ece9015, 0x3ed15fc0, 0x3ed423b1, 0x3ed6dc67, 0x3ed98a5c, 0x3edc2e00,
    0x3edec7bd, 0x3ee157f4, 0x3ee3df02, 0x3ee65d3c, 0x3ee8d2f8, 0x3eeb407e, 0x3eeda61a, 0x3ef00410,
    0x3ef25a9e, 0x3ef4aa08, 0x3ef6f282, 0x3ef93444, 0x3efb6f86, 0x3efda476, 0x3effd348, 0x3f00fe12,
    0x3f020f9e, 0x3f031e59, 0x3f042a59, 0x3f0533b0, 0x3f063a71, 0x3f073eac, 0x3f084072, 0x3f093fd4,
    0x3f0a3cdf, 0x3f0b37a2, 0x3f0c302d, 0x3f0d268b, 0x3f0e1acc, 0x3f0f0cfa, 0x3f0ffd21, 0x3f10eb4f,
    0x3f11d78d, 0x3f12c1e6, 0x3f13aa64, 0x3f149113, 0x3f1575fb, 0x3f165926, 0x3f173a9d, 0x3f181a68,
    0x3f18f891, 0x3f19d51e, 0x3f1ab01a, 0x3f1b8989, 0x3f1c6176, 0x3f1d37e7, 0x3f1e0ce3, 0x3f1ee06f,
    0x3f1fb293, 0x3f208357, 0x3f2152c0, 0x3f2220d2, 0x3f22ed96, 0x3f23b910, 0x3f248346, 0x3f254c3e,
    0x3f2613fd, 0x3f26da87, 0x3f279fe1, 0x3f286413, 0x3f29271e, 0x3f29e908, 0x3f2aa9d5, 0x3f2b698a,
    0x3f2c282d, 0x3f2ce5bf, 0x3f2da245, 0x3f2e5dc4, 0x3f2f183f, 0x3f2fd1b9, 0x3f308a38, 0x3f3141bf,
    0x3f31f84f, 0x3f32adee, 0x3f3362a0, 0x3f341665, 0x3f34c943, 0x3f357b3c, 0x3f362c54, 0x3f36dc8d,
    0x3f378bea, 0x3f383a6f, 0x3f38e81d, 0x3f3994f9, 0x3f3a4103, 0x3f3aec40, 0x3f3b96b2, 0x3f3c405a,
    0x3f3ce93d, 0x3f3d915c, 0x3f3e38b9, 0x3f3edf57, 0x3f3f8539, 0x3f402a5f, 0x3f40cece, 0x3f417287,
    0x3f42158a, 0x3f42b7dd, 0x3f435980, 0x3f43fa74, 0x3f449abd, 0x3f453a5b, 0x3f45d952, 0x3f4677a2,
    0x3f47154e, 0x3f47b257, 0x3f484ebf, 0x3f48ea88, 0x3f4985b4, 0x3f4a2044, 0x3f4aba3b, 0x3f4b5398,
    0x3f4bec5f, 0x3f4c8490, 0x3f4d1c2d, 0x3f4db338, 0x3f4e49b3, 0x3f4edf9e, 0x3f4f74fc, 0x3f5009cc,
    0x3f509e11, 0x3f5131ce, 0x3f51c502, 0x3f5257ae, 0x3f52e9d5, 0x3f537b77, 0x3f540c97, 0x3f549d34,
    0x3f552d50, 0x3f55bcee, 0x3f564c0c, 0x3f56daae, 0x3f5768d4, 0x3f57f67f, 0x3f5883b0, 0x3f591068,
    0x3f599caa, 0x3f5a2873, 0x3f5ab3c9, 0x3f5b3ea9, 0x3f5bc917, 0x3f5c5311, 0x3f5cdc9c, 0x3f5d65b5,
    0x3f5dee5e, 0x3f5e769a, 0x3f5efe68, 0x3f5f85c9, 0x3f600cbf, 0x3f60934a, 0x3f61196b, 0x3f619f23,
    0x3f622474, 0x3f62a95b, 0x3f632dde, 0x3f63b1fb, 0x3f6435b3, 0x3f64b907, 0x3f653bf8, 0x3f65be86,
    0x3f6640b3, 0x3f66c27f, 0x3f6743eb, 0x3f67c4f8, 0x3f6845a6, 0x3f68c5f6, 0x3f6945ea, 0x3f69c581,
    0x3f6a44bc, 0x3f6ac39c, 0x3f6b4222, 0x3f6bc04e, 0x3f6c3e21, 0x3f6cbb9c, 0x3f6d38c0, 0x3f6db58b,
    0x3f6e3201, 0x3f6eae22, 0x3f6f29ed, 0x3f6fa563, 0x3f702086, 0x3f709b55, 0x3f7115d2, 0x3f718ffc,
    0x3f7209d5, 0x3f72835d, 0x3f72fc95, 0x3f73757f, 0x3f73ee17, 0x3f746661, 0x3f74de5d, 0x3f75560b,
    0x3f75cd6b, 0x3f764481, 0x3f76bb49, 0x3f7731c5, 0x3f77a7f7, 0x3f781ddf, 0x3f78937b, 0x3f7908cf,
    0x3f797ddb, 0x3f79f29d, 0x3f7a6715, 0x3f7adb47, 0x3f7b4f33, 0x3f7bc2d9, 0x3f7c3635, 0x3f7ca94d,
    0x3f7d1c21, 0x3f7d8eaf, 0x3f7e00f9, 0x3f7e72ff, 0x3f7ee4c1, 0x3f7f563f, 0x3f7fc77b,
];

/// Convert one perceptual sRGB channel to rounded 8-bit linear-light PWM using
/// exactly the IEC 61966-2-1 transfer function's `f32` output boundaries.
#[inline]
fn srgb_to_linear_pwm8(srgb: f32) -> u8 {
    if srgb.is_nan() {
        return 0;
    }
    let srgb = clamp01(srgb);
    let bits = srgb.to_bits();
    let mut lo = 0;
    let mut hi = SRGB_TO_PWM_TRANSITIONS.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if bits < SRGB_TO_PWM_TRANSITIONS[mid] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo as u8
}

/// Encode sRGB pixels as linear PWM bytes in the configured wire order.
pub fn encode_ws281x(leds: &[Pixel], order: ColorOrder, out: &mut Vec<u8>) {
    out.clear();
    out.reserve(leds.len() * 3);
    for pixel in leds {
        let r = srgb_to_linear_pwm8(pixel.r);
        let g = srgb_to_linear_pwm8(pixel.g);
        let b = srgb_to_linear_pwm8(pixel.b);
        out.extend_from_slice(&order.apply(r, g, b));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_srgb_to_linear_pwm8(srgb: f32) -> u8 {
        let srgb = clamp01(srgb);
        let linear = if srgb <= 0.04045 {
            srgb / 12.92
        } else {
            libm::powf((srgb + 0.055) / 1.055, 2.4)
        };
        (linear * 255.0 + 0.5) as u8
    }

    fn gray(value: f32) -> Pixel {
        let mut pixel = Pixel::new();
        pixel.set_color(value, value, value);
        pixel
    }

    #[test]
    fn exact_srgb_eotf_vectors() {
        let leds = [
            gray(0.0),
            gray(0.1),
            gray(0.25),
            gray(0.5),
            gray(0.75),
            gray(1.0),
        ];
        let mut out = Vec::new();
        encode_ws281x(&leds, ColorOrder::Rgb, &mut out);
        assert_eq!(
            out,
            alloc::vec![
                0, 0, 0, 3, 3, 3, 13, 13, 13, 55, 55, 55, 133, 133, 133, 255, 255, 255,
            ]
        );
    }

    #[test]
    fn clamps_before_transfer_and_uses_grb_for_ws2815() {
        let mut pixel = Pixel::new();
        pixel.set_color(1.5, 0.5, -0.25);
        let mut out = Vec::new();
        encode_ws281x(&[pixel], Ws281xType::Ws2815.default_color_order(), &mut out);
        assert_eq!(out, alloc::vec![55, 255, 0]);
    }

    #[test]
    fn parses_all_supported_clockless_types() {
        assert_eq!(Ws281xType::parse("WS2811"), Some(Ws281xType::Ws2811));
        assert_eq!(Ws281xType::parse("ws2812b"), Some(Ws281xType::Ws2812));
        assert_eq!(Ws281xType::parse("WS2813"), Some(Ws281xType::Ws2813));
        assert_eq!(Ws281xType::parse("WS2815"), Some(Ws281xType::Ws2815));
        assert_eq!(Ws281xType::parse("SK6812"), Some(Ws281xType::Sk6812));
        assert_eq!(Ws281xType::parse("APA102"), None);
    }

    #[test]
    fn integer_search_matches_reference_at_every_transition() {
        assert_eq!(srgb_to_linear_pwm8(f32::NAN), 0);
        assert_eq!(srgb_to_linear_pwm8(f32::NEG_INFINITY), 0);
        assert_eq!(srgb_to_linear_pwm8(f32::INFINITY), 255);
        for &transition in &SRGB_TO_PWM_TRANSITIONS {
            for bits in transition.saturating_sub(2)..=transition.saturating_add(2) {
                let srgb = f32::from_bits(bits);
                assert_eq!(
                    srgb_to_linear_pwm8(srgb),
                    reference_srgb_to_linear_pwm8(srgb),
                    "mismatch at f32 bits {bits:#010x}"
                );
            }
        }
    }
}
