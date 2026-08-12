use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

const COLOR_DURATION_MS: u32 = 3_000;

/// Full-chassis lamp test: white, red, green, then blue, holding each color
/// for three seconds. The sequence restarts at white each time it is enabled.
#[derive(Default)]
pub struct LampTest {
    started_ms: Option<u32>,
}

impl Pattern for LampTest {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let started_ms = *self.started_ms.get_or_insert(info.millis);
        let phase = info.millis.wrapping_sub(started_ms) / COLOR_DURATION_MS % 4;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::test_support::{bootstrap, chassis};

    #[test]
    fn starts_white_and_advances_every_three_seconds() {
        let mut chassis = chassis();
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
}
