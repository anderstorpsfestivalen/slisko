use alloc::vec::Vec;

use crate::chassi::{CARD_PITCH, CHASSIS_HEIGHT, Chassi};
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

const SCROLL_PERIOD_MS: u32 = 60_000;
const WAVE_PERIOD_SECS: f32 = 24.0;
const WAVE_LENGTH: f32 = CARD_PITCH * 4.0;
const WAVE_AMPLITUDE: f32 = 45.0;

const COLORS: [(f32, f32, f32); 6] = [
    (0.894, 0.012, 0.012), // red
    (1.000, 0.549, 0.000), // orange
    (1.000, 0.929, 0.000), // yellow
    (0.000, 0.502, 0.149), // green
    (0.000, 0.302, 1.000), // blue
    (0.459, 0.027, 0.529), // violet
];

#[derive(Clone, Copy, Default)]
struct Sample {
    y: f32,
    wave_sin: f32,
    wave_cos: f32,
}

/// A six-stripe pride flag sampled in two-dimensional chassis space.
///
/// The flag scrolls vertically while a traveling sine wave bends its stripe
/// boundaries across the chassis. The spatial sine/cosine terms are cached at
/// bootstrap, leaving only one sine and cosine calculation per rendered frame.
#[derive(Default)]
pub struct Pride {
    samples: Vec<Sample>,
}

impl Pattern for Pride {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let scroll =
            (info.millis % SCROLL_PERIOD_MS) as f32 * (CHASSIS_HEIGHT / SCROLL_PERIOD_MS as f32);
        let wave_speed = core::f32::consts::TAU / WAVE_PERIOD_SECS;
        let wave_sin = utils::sin_full(info.secs, wave_speed);
        let wave_cos = utils::cos_full(info.secs, wave_speed);
        let stripe_height = CHASSIS_HEIGHT / COLORS.len() as f32;

        for (pixel, sample) in c.leds.iter_mut().zip(&self.samples) {
            let wave = WAVE_AMPLITUDE * (sample.wave_sin * wave_cos + sample.wave_cos * wave_sin);
            let stripe = (((sample.y + scroll + wave + CHASSIS_HEIGHT) / stripe_height) as usize)
                % COLORS.len();
            let (r, g, b) = COLORS[stripe];
            pixel.set_clamped(r, g, b);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "pride",
            category: "global",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, _ctx: &mut BootstrapCtx) {
        self.samples = c
            .chassis_positions()
            .into_iter()
            .map(|position| {
                let phase = position.x * (core::f32::consts::TAU / WAVE_LENGTH);
                Sample {
                    y: position.y,
                    wave_sin: utils::sin_full(phase, 1.0),
                    wave_cos: utils::cos_full(phase, 1.0),
                }
            })
            .collect();
    }
}
