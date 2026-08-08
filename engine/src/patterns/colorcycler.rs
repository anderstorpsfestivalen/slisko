//! Ported from `patterns/colorcycler.go`.

use crate::chassi::Chassi;
use crate::color::hsv;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

/// Cycles all LEDs through the hue wheel (the Go `Colorcycler` pattern).
#[derive(Default)]
pub struct Colorcycler {
    color: (f32, f32, f32),
}

impl Pattern for Colorcycler {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let v = utils::sin(info.secs, 0.2) * 360.0;
        self.color = hsv(v, 1.0, 1.0);
        let (r, g, b) = self.color;
        for p in &mut c.leds {
            p.set_clamped(r, g, b);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "colorcycler",
            category: "global",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {
        // Matches the Go bootstrap seed color (cosmetic; overwritten each frame).
        self.color = (0.313725, 0.478431, 0.721569);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::{Chassi, LineCardSpec};
    use crate::pixel::Position;

    static POS: &[Position] = &[Position {
        x: 0.0,
        y: 0.0,
        size: 1.0,
    }];
    static SPECS: &[LineCardSpec] = &[LineCardSpec {
        name: "C",
        image: "",
        active: true,
        positions: POS,
        link: &[],
        status: None,
        labeled: &[],
    }];

    #[test]
    fn at_t0_hue_is_red_ish() {
        // sin(0,0.2)=0.5 -> hue 180 -> cyan. Just assert it set *something* valid.
        let mut c = Chassi::from_specs(SPECS);
        let mut p = Colorcycler::default();
        let mut rng = crate::faker::Rng::new(1);
        let mut ctx = crate::pattern::BootstrapCtx {
            rng: &mut rng,
            intensity: 1.0,
        };
        p.bootstrap(&c, &mut ctx);
        p.render(
            &RenderInfo {
                secs: 0.0,
                frame: 0,
            },
            &mut c,
        );
        let (r, g, b) = p.color;
        assert!((0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b));
        // hue 180 = cyan: r≈0, g≈1, b≈1
        assert!(r < 0.01 && g > 0.99 && b > 0.99);
    }
}
