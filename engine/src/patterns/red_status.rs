use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

/// Lights all status LEDs red.
#[derive(Default)]
pub struct RedStatus;

impl Pattern for RedStatus {
    fn render(&mut self, _info: &RenderInfo, c: &mut Chassi) {
        let idxs = c.status_leds().to_vec();
        for i in idxs {
            c.leds[i].set_clamped(1.0, 0.3, 0.0);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "redstatus",
            category: "status",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}
