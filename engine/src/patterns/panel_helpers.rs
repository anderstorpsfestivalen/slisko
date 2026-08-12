use alloc::boxed::Box;

use crate::chassi::Chassi;
use crate::faker::{Fake, RandomBlinker, RandomInterval, Rng};
use crate::pattern::BootstrapCtx;

pub(crate) fn random_interval(
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

pub(crate) fn set_all(c: &mut Chassi, indices: &[usize], r: f32, g: f32, b: f32) {
    for &index in indices {
        c.leds[index].set_clamped(r, g, b);
    }
}
