//! Simple global patterns: `Strobe` and `Snake` (ported from
//! `patterns/strobe.go`, `patterns/snake.go`).

use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

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
