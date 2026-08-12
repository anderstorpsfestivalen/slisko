use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

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
    use crate::patterns::test_support::chassis;

    #[test]
    fn clears_every_pixel() {
        let mut chassis = chassis();
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
