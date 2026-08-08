use alloc::vec::Vec;

use crate::pixel::Pixel;

const STEP: u16 = 200;
const RED_START: u16 = 6400;
const GREEN_START: u16 = 1000;
const BLUE_START: u16 = 1000;

const RED: &[u8] = &[
    0xFF, 0xFF, 0xF9, 0xF5, 0xF0, 0xED, 0xE9, 0xE6, 0xE3, 0xE0, 0xDD, 0xDA, 0xD8, 0xD6, 0xD3, 0xD1,
    0xCF, 0xCE, 0xCC, 0xCA, 0xC9, 0xC7, 0xC6, 0xC4, 0xC3, 0xC2, 0xC1, 0xC0, 0xBF, 0xBE, 0xBD, 0xBC,
    0xBB, 0xBA, 0xB9, 0xB8, 0xB7, 0xB7, 0xB6, 0xB5, 0xB5, 0xB4, 0xB3, 0xB3, 0xB2, 0xB2, 0xB1, 0xB1,
    0xB0, 0xAF, 0xAF, 0xAF, 0xAE, 0xAE, 0xAD, 0xAD, 0xAC, 0xAC, 0xAC, 0xAB, 0xAB, 0xAA, 0xAA, 0xAA,
    0xA9, 0xA9, 0xA9, 0xA9, 0xA8, 0xA8, 0xA8, 0xA7, 0xA7, 0xA7, 0xA7, 0xA6, 0xA6, 0xA6, 0xA6, 0xA5,
    0xA5, 0xA5, 0xA5, 0xA4, 0xA4, 0xA4, 0xA4, 0xA4, 0xA3, 0xA3, 0xA3, 0xA3, 0xA3, 0xA3, 0xA2, 0xA2,
    0xA2, 0xA2, 0xA2, 0xA2, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xA1, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0,
    0xA0, 0xA0, 0xA0, 0x9F, 0x9F, 0x9F, 0x9F,
];

const GREEN: &[u8] = &[
    0x38, 0x53, 0x65, 0x73, 0x7E, 0x89, 0x93, 0x9D, 0xA5, 0xAD, 0xB4, 0xBB, 0xC1, 0xC7, 0xCC, 0xD1,
    0xD5, 0xD9, 0xDD, 0xE1, 0xE4, 0xE8, 0xEB, 0xEE, 0xF0, 0xF3, 0xF5, 0xFF, 0xFF, 0xF6, 0xF3, 0xF1,
    0xEF, 0xED, 0xEB, 0xE9, 0xE7, 0xE6, 0xE4, 0xE3, 0xE1, 0xE0, 0xDF, 0xDD, 0xDC, 0xDB, 0xDA, 0xD9,
    0xD8, 0xD8, 0xD7, 0xD6, 0xD5, 0xD4, 0xD4, 0xD3, 0xD2, 0xD2, 0xD1, 0xD1, 0xD0, 0xD0, 0xCF, 0xCF,
    0xCE, 0xCE, 0xCD, 0xCD, 0xCC, 0xCC, 0xCC, 0xCB, 0xCB, 0xCA, 0xCA, 0xCA, 0xC9, 0xC9, 0xC9, 0xC9,
    0xC8, 0xC8, 0xC8, 0xC7, 0xC7, 0xC7, 0xC7, 0xC6, 0xC6, 0xC6, 0xC6, 0xC6, 0xC5, 0xC5, 0xC5, 0xC5,
    0xC5, 0xC4, 0xC4, 0xC4, 0xC4, 0xC4, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC2, 0xC2, 0xC2,
    0xC2, 0xC2, 0xC2, 0xC2, 0xC1, 0xC1, 0xC1, 0xC1, 0xC1, 0xC1, 0xC1, 0xC1, 0xC1, 0xC0, 0xC0, 0xC0,
    0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xC0, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF, 0xBF,
    0xBF, 0xBF,
];

const BLUE: &[u8] = &[
    0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x2C, 0x3F, 0x4F, 0x5E, 0x6B, 0x78, 0x84, 0x8F, 0x99, 0xA3,
    0xAD, 0xB6, 0xBE, 0xC6, 0xCE, 0xD5, 0xDC, 0xE3, 0xE9, 0xEF, 0xF5, 0xFF, 0xFF,
];

/// Integer-only temperature interpolation with the original operation order.
pub(super) fn to_rgb_fast(mut kelvin: u16) -> (u8, u8, u8) {
    if kelvin == 6500 {
        return (255, 255, 255);
    }
    kelvin = kelvin.clamp(1000, 29999);

    let (mut index, mut ratio) = interpolation(kelvin, GREEN_START);
    let green = interpolate(GREEN, index, ratio);
    if kelvin < 6500 {
        (index, ratio) = interpolation(kelvin, BLUE_START);
        return (255, green, interpolate(BLUE, index, ratio));
    }
    (index, ratio) = interpolation(kelvin, RED_START);
    (interpolate(RED, index, ratio), green, 255)
}

fn interpolation(kelvin: u16, start: u16) -> (usize, u32) {
    let distance = kelvin - start;
    (
        usize::from(distance / STEP),
        u32::from((distance % STEP) * 255 / STEP),
    )
}

fn interpolate(table: &[u8], index: usize, ratio: u32) -> u8 {
    ((ratio * u32::from(table[index]) + (255 - ratio) * u32::from(table[index + 1])) / 255) as u8
}

/// Temperature where APA102 color correction is disabled.
pub const APA102_NEUTRAL_TEMPERATURE: u16 = 6500;

/// Maximum combined intensity from the 8-bit channel and 5-bit global PWMs.
const MAX_OUT: u16 = 0x1EE1;

/// Device-specific APA102 color options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Apa102Options {
    /// Maximum output intensity on the source driver's perceptual scale.
    pub intensity: u8,
    /// Corrected white point in Kelvin. `6500` is neutral.
    pub temperature: u16,
    /// Use the full combined 13-bit channel/global-PWM range.
    pub global_pwm: bool,
}

impl Apa102Options {
    pub const DEFAULT: Self = Self {
        intensity: 255,
        temperature: 5000,
        global_pwm: true,
    };
}

impl Default for Apa102Options {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Stateful APA102 frame encoder with a cached device-options LUT.
pub struct Apa102Encoder {
    options: Apa102Options,
    lut: Lut,
}

impl Apa102Encoder {
    pub fn new(options: Apa102Options) -> Self {
        Self {
            options,
            lut: Lut::new(options),
        }
    }

    pub fn options(&self) -> Apa102Options {
        self.options
    }

    /// Replace the device options and rebuild the cached lookup table when any
    /// option changes.
    pub fn set_options(&mut self, options: Apa102Options) {
        if options != self.options {
            self.options = options;
            self.lut.rebuild(options);
        }
    }

    /// Encode pixels as one complete APA102 SPI transaction: four zero start
    /// bytes, BGR pixel payloads, and `pixel_count / 16 + 1` end clocks.
    pub fn encode(&self, pixels: &[Pixel], out: &mut Vec<u8>) {
        let end_len = pixels.len() / 16 + 1;
        out.clear();
        out.reserve(4 + pixels.len() * 4 + end_len);
        out.extend_from_slice(&[0, 0, 0, 0]);

        for pixel in pixels {
            let [r, g, b] = pixel.to_srgb8();
            let r = self.lut.r[r as usize];
            let g = self.lut.g[g as usize];
            let b = self.lut.b[b as usize];
            if !self.options.global_pwm {
                out.extend_from_slice(&[0xFF, b as u8, g as u8, r as u8]);
                continue;
            }

            encode_global_pixel(out, r, g, b);
        }

        out.resize(out.len() + end_len, 0xFF);
    }
}

/// Match the Go driver's bitwise-OR thresholds exactly. The selected global
/// brightness is the smallest of 1, 2, 4, or 31 that can represent the three
/// 13-bit channel values.
fn encode_global_pixel(out: &mut Vec<u8>, r: u16, g: u16, b: u16) {
    let magnitude = r | g | b;
    if magnitude <= 255 {
        out.extend_from_slice(&[0xE1, b as u8, g as u8, r as u8]);
    } else if magnitude <= 511 {
        out.extend_from_slice(&[0xE2, (b / 2) as u8, (g / 2) as u8, (r / 2) as u8]);
    } else if magnitude <= 1023 {
        out.extend_from_slice(&[
            0xE4,
            ((b + 2) / 4) as u8,
            ((g + 2) / 4) as u8,
            ((r + 2) / 4) as u8,
        ]);
    } else {
        out.extend_from_slice(&[
            0xFF,
            ((b + 15) / 31) as u8,
            ((g + 15) / 31) as u8,
            ((r + 15) / 31) as u8,
        ]);
    }
}

impl Default for Apa102Encoder {
    fn default() -> Self {
        Self::new(Apa102Options::default())
    }
}

struct Lut {
    r: [u16; 256],
    g: [u16; 256],
    b: [u16; 256],
}

impl Lut {
    fn new(options: Apa102Options) -> Self {
        let mut lut = Self {
            r: [0; 256],
            g: [0; 256],
            b: [0; 256],
        };
        lut.rebuild(options);
        lut
    }

    fn rebuild(&mut self, options: Apa102Options) {
        let (tr, tg, tb) = to_rgb_fast(options.temperature);
        if !options.global_pwm {
            let max_r = (u32::from(options.intensity) * u32::from(tr) + 127) / 255;
            let max_g = (u32::from(options.intensity) * u32::from(tg) + 127) / 255;
            let max_b = (u32::from(options.intensity) * u32::from(tb) + 127) / 255;
            for value in 0..256u32 {
                self.r[value as usize] = ((value * max_r + 127) / 255) as u16;
                self.g[value as usize] = ((value * max_g + 127) / 255) as u16;
                self.b[value as usize] = ((value * max_b + 127) / 255) as u16;
            }
            return;
        }

        let max_r =
            (u32::from(MAX_OUT) * u32::from(options.intensity) * u32::from(tr) + 127 * 127) / 65025;
        let max_g =
            (u32::from(MAX_OUT) * u32::from(options.intensity) * u32::from(tg) + 127 * 127) / 65025;
        let max_b =
            (u32::from(MAX_OUT) * u32::from(options.intensity) * u32::from(tb) + 127 * 127) / 65025;
        for value in 0..=255u8 {
            self.r[value as usize] = ramp(value, max_r as u16);
        }
        if max_g == max_r {
            self.g = self.r;
        } else {
            for value in 0..=255u8 {
                self.g[value as usize] = ramp(value, max_g as u16);
            }
        }
        if max_b == max_r {
            self.b = self.r;
        } else if max_b == max_g {
            self.b = self.g;
        } else {
            for value in 0..=255u8 {
                self.b[value as usize] = ramp(value, max_b as u16);
            }
        }
    }
}

/// Reverse-lightness cubic ramp ported with the Go driver's integer operation
/// order intact.
fn ramp(level: u8, max: u16) -> u16 {
    if level == 0 {
        return 0;
    }
    let linear_cutoff = u32::from(max + 50) / 100;
    let mut level = u32::from(level);
    if level < linear_cutoff {
        return level as u16;
    }
    level -= linear_cutoff;
    let in_range = 255 - linear_cutoff;
    let out_range = u32::from(max) - linear_cutoff;
    let offset = in_range >> 1;
    let y = (level * level * level + offset) / in_range;
    ((y * out_range + offset * offset) / in_range / in_range + linear_cutoff) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixels(bytes: &[[u8; 3]]) -> Vec<Pixel> {
        bytes
            .iter()
            .map(|[r, g, b]| {
                let mut pixel = Pixel::new();
                pixel.set_color(
                    f32::from(*r) / 255.0,
                    f32::from(*g) / 255.0,
                    f32::from(*b) / 255.0,
                );
                pixel
            })
            .collect()
    }

    fn encode(options: Apa102Options, bytes: &[[u8; 3]]) -> Vec<u8> {
        let mut out = Vec::new();
        Apa102Encoder::new(options).encode(&pixels(bytes), &mut out);
        out
    }

    #[test]
    fn defaults_match_the_reference_driver() {
        assert_eq!(
            Apa102Options::default(),
            Apa102Options {
                intensity: 255,
                temperature: 5000,
                global_pwm: true
            }
        );
        assert_eq!(APA102_NEUTRAL_TEMPERATURE, 6500);
    }

    #[test]
    fn temperature_boundaries_neutral_and_interpolation_match_go() {
        assert_eq!(to_rgb_fast(999), (255, 83, 0));
        assert_eq!(to_rgb_fast(1000), (255, 83, 0));
        assert_eq!(to_rgb_fast(1100), (255, 69, 0));
        assert_eq!(to_rgb_fast(5000), (255, 232, 213));
        assert_eq!(to_rgb_fast(6500), (255, 255, 255));
        assert_eq!(to_rgb_fast(29999), (159, 191, 255));
        assert_eq!(to_rgb_fast(30000), (159, 191, 255));
    }

    #[test]
    fn neutral_temperature_golden_vector_matches_go() {
        let input = [
            [255, 255, 255],
            [254, 254, 254],
            [240, 240, 240],
            [128, 128, 128],
            [128, 0, 0],
            [0, 128, 0],
            [0, 0, 128],
            [0, 0, 16],
            [0, 0, 1],
            [0, 0, 0],
        ];
        let actual = encode(
            Apa102Options {
                intensity: 255,
                temperature: 6500,
                global_pwm: true,
            },
            &input,
        );
        assert_eq!(
            actual,
            alloc::vec![
                0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB, 0xFB, 0xFB, 0xFF, 0xC4, 0xC4, 0xC4,
                0xE1, 0xF8, 0xF8, 0xF8, 0xE1, 0, 0, 0xF8, 0xE1, 0, 0xF8, 0, 0xE1, 0xF8, 0, 0, 0xE1,
                0x10, 0, 0, 0xE1, 1, 0, 0, 0xE1, 0, 0, 0, 0xFF,
            ]
        );
    }

    #[test]
    fn disabled_global_pwm_temperature_vector_matches_go() {
        let input = [
            [255, 255, 255],
            [254, 254, 254],
            [240, 240, 240],
            [128, 128, 128],
            [128, 0, 0],
            [0, 128, 0],
            [0, 0, 128],
            [0, 0, 16],
            [0, 0, 1],
            [0, 0, 0],
        ];
        let actual = encode(
            Apa102Options {
                intensity: 255,
                temperature: 5000,
                global_pwm: false,
            },
            &input,
        );
        assert_eq!(
            actual,
            alloc::vec![
                0, 0, 0, 0, 0xFF, 0xD5, 0xE8, 0xFF, 0xFF, 0xD4, 0xE7, 0xFE, 0xFF, 0xC8, 0xDA, 0xF0,
                0xFF, 0x6B, 0x74, 0x80, 0xFF, 0, 0, 0x80, 0xFF, 0, 0x74, 0, 0xFF, 0x6B, 0, 0, 0xFF,
                0x0D, 0, 0, 0xFF, 1, 0, 0, 0xFF, 0, 0, 0, 0xFF,
            ]
        );
    }

    #[test]
    fn intensity_vectors_cover_zero_half_and_full() {
        let input = [
            [0, 0, 0],
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [128, 128, 128],
            [255, 255, 255],
        ];
        let zero = encode(
            Apa102Options {
                intensity: 0,
                temperature: 6500,
                global_pwm: true,
            },
            &input,
        );
        assert!(zero[4..28].chunks_exact(4).all(|p| p == [0xE1, 0, 0, 0]));

        let half = encode(
            Apa102Options {
                intensity: 128,
                temperature: 6500,
                global_pwm: true,
            },
            &input,
        );
        assert_eq!(
            &half[4..28],
            &[
                0xE1, 0, 0, 0, 0xFF, 0, 0, 128, 0xFF, 0, 128, 0, 0xFF, 128, 0, 0, 0xE2, 154, 154,
                154, 0xFF, 128, 128, 128,
            ]
        );

        let full = encode(
            Apa102Options {
                intensity: 255,
                temperature: 6500,
                global_pwm: false,
            },
            &input,
        );
        assert_eq!(
            &full[4..28],
            &[
                0xFF, 0, 0, 0, 0xFF, 0, 0, 255, 0xFF, 0, 255, 0, 0xFF, 255, 0, 0, 0xFF, 128, 128,
                128, 0xFF, 255, 255, 255,
            ]
        );
    }

    #[test]
    fn default_5000k_global_pwm_corrects_primaries_in_bgr_order() {
        let input = [
            [0, 0, 0],
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [255, 255, 255],
        ];
        let actual = encode(Apa102Options::default(), &input);
        assert_eq!(
            &actual[4..24],
            &[
                0xE1, 0, 0, 0, 0xFF, 0, 0, 255, 0xFF, 0, 232, 0, 0xFF, 213, 0, 0, 0xFF, 213, 232,
                255,
            ]
        );
    }

    #[test]
    fn disabled_global_pwm_at_half_intensity_is_linear() {
        let actual = encode(
            Apa102Options {
                intensity: 128,
                temperature: 6500,
                global_pwm: false,
            },
            &[[0, 0, 0], [128, 128, 128], [255, 255, 255]],
        );
        assert_eq!(
            &actual[4..16],
            &[0xFF, 0, 0, 0, 0xFF, 64, 64, 64, 0xFF, 128, 128, 128]
        );
    }

    #[test]
    fn global_brightness_thresholds_and_rounding_match_go() {
        let cases = [
            (255, [0xE1, 255, 255, 255]),
            (256, [0xE2, 128, 128, 128]),
            (511, [0xE2, 255, 255, 255]),
            (512, [0xE4, 128, 128, 128]),
            (1023, [0xE4, 0, 0, 0]),
            (1024, [0xFF, 33, 33, 33]),
        ];
        for (value, expected) in cases {
            let mut out = Vec::new();
            encode_global_pixel(&mut out, value, value, value);
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn setter_rebuilds_the_cached_lut() {
        let white = pixels(&[[255, 255, 255]]);
        let mut encoder = Apa102Encoder::default();
        let mut out = Vec::new();
        encoder.encode(&white, &mut out);
        assert_eq!(&out[4..8], &[0xFF, 0xD5, 0xE8, 0xFF]);
        encoder.set_options(Apa102Options {
            intensity: 255,
            temperature: 6500,
            global_pwm: false,
        });
        encoder.encode(&white, &mut out);
        assert_eq!(&out[4..8], &[0xFF, 255, 255, 255]);
    }

    #[test]
    fn exact_footer_layout_matches_go() {
        for count in [0, 1, 15, 16, 17, 31, 32] {
            let pixels = alloc::vec![Pixel::new(); count];
            let mut out = Vec::new();
            Apa102Encoder::default().encode(&pixels, &mut out);
            let footer = count / 16 + 1;
            assert_eq!(out.len(), 4 + count * 4 + footer);
            assert!(out[out.len() - footer..].iter().all(|byte| *byte == 0xFF));
        }
    }

    #[test]
    fn cubic_ramp_reference_points_match_go() {
        assert_eq!(ramp(0, MAX_OUT), 0);
        assert_eq!(ramp(127, 255), 0x21);
        assert_eq!(ramp(255, 255), 0xFF);
        assert_eq!(ramp(255, MAX_OUT), MAX_OUT);
    }
}
