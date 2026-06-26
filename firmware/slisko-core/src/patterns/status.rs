//! Ported from `patterns/status.go`.

use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

/// Lights all status LEDs green (the Go `GreenStatus`).
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

/// Lights all status LEDs red (the Go `RedStatus`).
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
