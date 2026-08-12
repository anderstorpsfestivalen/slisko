use crate::chassi::{Chassi, LineCardSpec};
use crate::faker::Rng;
use crate::pattern::{BootstrapCtx, Pattern};
use crate::pixel::Position;

static POSITIONS: &[Position] = &[
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
    Position {
        x: 2.0,
        y: 0.0,
        size: 1.0,
    },
    Position {
        x: 3.0,
        y: 0.0,
        size: 1.0,
    },
];

static SPECS: &[LineCardSpec] = &[LineCardSpec {
    name: "test",
    image: "",
    active: true,
    positions: POSITIONS,
    link: &[],
    status: None,
    labeled: &[],
}];

pub fn chassis() -> Chassi {
    Chassi::from_specs(SPECS)
}

pub fn bootstrap(pattern: &mut dyn Pattern, chassis: &Chassi) {
    let mut rng = Rng::new(1);
    pattern.bootstrap(
        chassis,
        &mut BootstrapCtx {
            rng: &mut rng,
            intensity: 1.0,
        },
    );
}
