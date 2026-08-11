//! Full-chassis patterns, including the original `Strobe` and `Snake` ports
//! plus the hardware-control patterns used by the ASR9000 buttons.

use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::color::hsv;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

const RAINBOW_PERIOD_MS: u32 = 30_000;
const LAMP_COLOR_MS: u32 = 3_000;

/// Full-strand white strobe (mirrors `Strobe`).
#[derive(Default)]
pub struct Strobe;

impl Pattern for Strobe {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let v = utils::square(utils::sin_full(info.secs, 100.0));
        for p in &mut c.leds {
            p.set_clamped(v, v, v);
        }
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "strobe",
            category: "global",
        }
    }
    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}

/// A single lit pixel sweeping the strand (mirrors `Snake`).
#[derive(Default)]
pub struct Snake;

impl Pattern for Snake {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let n = c.leds.len();
        // The value is non-negative, so conversion is the same as floorf.
        let lit = (utils::sin(info.secs, 1.0) * n as f32) as usize;
        for (m, p) in c.leds.iter_mut().enumerate() {
            if m == lit {
                p.set_clamped(1.0, 1.0, 0.5);
            } else {
                p.set_clamped(0.0, 0.0, 0.0);
            }
        }
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "snake",
            category: "global",
        }
    }
    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}

/// A full hue wheel spread across the logical strand and slowly travelling
/// through it. One complete revolution takes 30 seconds.
#[derive(Default)]
pub struct Rainbow {
    base_hues: Vec<f32>,
}

impl Pattern for Rainbow {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let shift = (info.millis % RAINBOW_PERIOD_MS) as f32 * (360.0 / RAINBOW_PERIOD_MS as f32);
        for (pixel, &base_hue) in c.leds.iter_mut().zip(&self.base_hues) {
            let mut hue = base_hue + shift;
            if hue >= 360.0 {
                hue -= 360.0;
            }
            let (r, g, b) = hsv(hue, 1.0, 1.0);
            pixel.set_clamped(r, g, b);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "rainbow",
            category: "global",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, _ctx: &mut BootstrapCtx) {
        let count = c.leds.len().max(1) as f32;
        self.base_hues = (0..c.leds.len())
            .map(|index| index as f32 * (360.0 / count))
            .collect();
    }
}

/// Full-chassis lamp test: white, red, green, then blue, holding each color
/// for three seconds. The sequence restarts at white each time it is enabled.
#[derive(Default)]
pub struct LampTest {
    started_ms: Option<u32>,
}

impl Pattern for LampTest {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let started_ms = *self.started_ms.get_or_insert(info.millis);
        let phase = info.millis.wrapping_sub(started_ms) / LAMP_COLOR_MS % 4;
        let color = match phase {
            0 => (1.0, 1.0, 1.0),
            1 => (1.0, 0.0, 0.0),
            2 => (0.0, 1.0, 0.0),
            _ => (0.0, 0.0, 1.0),
        };
        for pixel in &mut c.leds {
            pixel.set_clamped(color.0, color.1, color.2);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "lamp-test",
            category: "global",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {
        self.started_ms = None;
    }
}

/// Continuously paints the full logical chassis black.
#[derive(Default)]
pub struct Blackout;

impl Pattern for Blackout {
    fn render(&mut self, _info: &RenderInfo, c: &mut Chassi) {
        for pixel in &mut c.leds {
            pixel.set_clamped(0.0, 0.0, 0.0);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "blackout",
            category: "global",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::LineCardSpec;
    use crate::faker::Rng;
    use crate::pixel::Position;

    static POSITIONS: &[Position] = &[
        Position {
            x: 0.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 1.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 2.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 3.0,
            y: 0.0,
            size: 1.0,
        },
    ];
    static SPECS: &[LineCardSpec] = &[LineCardSpec {
        name: "test",
        image: "",
        active: true,
        positions: POSITIONS,
        link: &[],
        status: None,
        labeled: &[],
    }];

    fn bootstrap(pattern: &mut dyn Pattern, chassis: &Chassi) {
        let mut rng = Rng::new(1);
        pattern.bootstrap(
            chassis,
            &mut BootstrapCtx {
                rng: &mut rng,
                intensity: 1.0,
            },
        );
    }

    #[test]
    fn rainbow_is_spatial_and_moves_slowly() {
        let mut chassis = Chassi::from_specs(SPECS);
        let mut pattern = Rainbow::default();
        bootstrap(&mut pattern, &chassis);
        pattern.render(
            &RenderInfo {
                millis: 0,
                ..RenderInfo::default()
            },
            &mut chassis,
        );
        let initial = chassis.leds.clone();
        assert_ne!(initial[0].to_srgb8(), initial[1].to_srgb8());

        pattern.render(
            &RenderInfo {
                millis: RAINBOW_PERIOD_MS / 4,
                ..RenderInfo::default()
            },
            &mut chassis,
        );
        assert_eq!(chassis.leds[0].to_srgb8(), initial[1].to_srgb8());
    }

    #[test]
    fn lamp_test_starts_white_and_advances_every_three_seconds() {
        let mut chassis = Chassi::from_specs(SPECS);
        let mut pattern = LampTest::default();
        bootstrap(&mut pattern, &chassis);
        for (millis, expected) in [
            (40_000, [255, 255, 255]),
            (42_999, [255, 255, 255]),
            (43_000, [255, 0, 0]),
            (46_000, [0, 255, 0]),
            (49_000, [0, 0, 255]),
            (52_000, [255, 255, 255]),
        ] {
            pattern.render(
                &RenderInfo {
                    millis,
                    ..RenderInfo::default()
                },
                &mut chassis,
            );
            assert_eq!(chassis.leds[0].to_srgb8(), expected);
        }
    }

    #[test]
    fn blackout_clears_every_pixel() {
        let mut chassis = Chassi::from_specs(SPECS);
        for pixel in &mut chassis.leds {
            pixel.set_clamped(1.0, 1.0, 1.0);
        }
        Blackout.render(&RenderInfo::default(), &mut chassis);
        assert!(
            chassis
                .leds
                .iter()
                .all(|pixel| pixel.to_srgb8() == [0, 0, 0])
        );
    }
}
