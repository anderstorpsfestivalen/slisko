use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

/// Full-strand white strobe.
#[derive(Default)]
pub struct Strobe;

impl Pattern for Strobe {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let value = utils::square(utils::sin_full(info.secs, 100.0));
        for pixel in &mut c.leds {
            pixel.set_clamped(value, value, value);
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
