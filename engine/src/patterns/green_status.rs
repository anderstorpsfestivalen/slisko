use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

/// Lights all status LEDs green.
#[derive(Default)]
pub struct GreenStatus;

impl Pattern for GreenStatus {
    fn render(&mut self, _info: &RenderInfo, c: &mut Chassi) {
        let idxs = c.status_leds().to_vec();
        for i in idxs {
            c.leds[i].set_clamped(0.3, 1.0, 0.0);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "greenstatus",
            category: "status",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}
