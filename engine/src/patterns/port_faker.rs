use alloc::boxed::Box;

use crate::faker::{Fake, RandomBlinker, RandomInterval, Rng};
use crate::pattern::BootstrapCtx;

/// One port driven by a `RandomInterval(RandomBlinker)` faker.
pub(crate) struct PortFaker {
    pub faker: Box<dyn Fake + Send>,
    pub port: usize,
}

/// The standard port faker used by both chassis families.
pub(crate) fn standard_faker(
    min_interval: f32,
    max_interval: f32,
    min_blink: f32,
    max_blink: f32,
    ctx: &mut BootstrapCtx,
) -> Box<dyn Fake + Send> {
    let blinker = RandomBlinker::new(15.0, 40.0, 1.0, 10.0, 0, Rng::new(ctx.rng.next_seed()));
    Box::new(RandomInterval::new(
        min_interval,
        max_interval,
        min_blink,
        max_blink,
        Box::new(blinker),
        0,
        Rng::new(ctx.rng.next_seed()),
    ))
}
