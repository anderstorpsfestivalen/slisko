use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

/// A single lit pixel sweeping the strand.
#[derive(Default)]
pub struct Snake;

impl Pattern for Snake {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let n = c.leds.len();
        let lit = (utils::sin(info.secs, 1.0) * n as f32) as usize;
        for (index, pixel) in c.leds.iter_mut().enumerate() {
            if index == lit {
                pixel.set_clamped(1.0, 1.0, 0.5);
            } else {
                pixel.set_clamped(0.0, 0.0, 0.0);
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
