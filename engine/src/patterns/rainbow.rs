use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::color::hsv;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

const PERIOD_MS: u32 = 30_000;

/// A full hue wheel spread across the logical strand and slowly travelling
/// through it. One complete revolution takes 30 seconds.
#[derive(Default)]
pub struct Rainbow {
    base_hues: Vec<f32>,
}

impl Pattern for Rainbow {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let shift = (info.millis % PERIOD_MS) as f32 * (360.0 / PERIOD_MS as f32);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::test_support::{bootstrap, chassis};

    #[test]
    fn is_spatial_and_moves_slowly() {
        let mut chassis = chassis();
        let mut pattern = Rainbow::default();
        bootstrap(&mut pattern, &chassis);
        pattern.render(&RenderInfo::default(), &mut chassis);
        let initial = chassis.leds.clone();
        assert_ne!(initial[0].to_srgb8(), initial[1].to_srgb8());

        pattern.render(
            &RenderInfo {
                millis: PERIOD_MS / 4,
                ..RenderInfo::default()
            },
            &mut chassis,
        );
        assert_eq!(chassis.leds[0].to_srgb8(), initial[1].to_srgb8());
    }
}
