//! Ported from `patterns/static.go`.

use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

/// Fills every LED with a fixed orange (the Go `Static` pattern).
#[derive(Default)]
pub struct Static;

impl Pattern for Static {
    fn render(&mut self, _info: &RenderInfo, c: &mut Chassi) {
        for p in &mut c.leds {
            p.set_clamped(1.0, 0.5, 0.5);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "static",
            category: "global",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::{Chassi, LineCardSpec};
    use crate::pixel::Position;

    static POS: &[Position] = &[
        Position {
            x: 0.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 1.0,
            y: 0.0,
            size: 1.0,
        },
    ];
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
    fn fills_all_leds() {
        let mut c = Chassi::from_specs(SPECS);
        let mut p = Static;
        let mut rng = crate::faker::Rng::new(1);
        let mut ctx = crate::pattern::BootstrapCtx {
            rng: &mut rng,
            intensity: 1.0,
        };
        p.bootstrap(&c, &mut ctx);
        p.render(&RenderInfo::default(), &mut c);
        for px in &c.leds {
            assert_eq!(px.to_rgb8(), [255, 127, 127]);
        }
    }
}
