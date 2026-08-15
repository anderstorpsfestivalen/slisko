//! Ported from `patterns/blinkstyle.go`.
//!
//! A [`BlinkStyle`] describes how a linecard's ports blink; [`PortState`] is one
//! port's runtime state (a faker + target pixel index + color), built by
//! [`BlinkStyle::create_port`]. Configured durations are `f32` seconds and
//! runtime deadlines are integer milliseconds (Go used `time.Duration`).

use alloc::boxed::Box;

use crate::faker::{Fake, RandomBlinker, RandomInterval, Rng};
use crate::pattern::BootstrapCtx;
use crate::pixel::Pixel;
use crate::utils;

/// RGB multipliers (0.0..1.0), mirrors `ColorStyle`.
#[derive(Clone, Copy, Debug)]
pub struct ColorStyle {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// Mirrors `BlinkStyle`. Timing fields are seconds.
#[derive(Clone, Copy, Debug)]
pub struct BlinkStyle {
    pub min_interval: f32,
    pub max_interval: f32,
    pub min_blink: f32,
    pub max_blink: f32,
    pub min_blinks: f32,
    pub max_blinks: f32,
    pub min_cycle: f32,
    pub max_cycle: f32,
    pub slow_color: ColorStyle,
    pub fast_color: ColorStyle,
    pub dead_color: ColorStyle,
    pub dead_port_chance: f32,
    pub slow_speed_chance: f32,
}

/// One port's runtime state (mirrors `PortState`).
pub struct PortState {
    faker: Option<Box<dyn Fake + Send>>,
    port: usize,
    style: ColorStyle,
    is_dead: bool,
}

impl PortState {
    /// Render this port into the strand (mirrors `PortState.Render`).
    pub fn render(&mut self, leds: &mut [Pixel], now: u32) {
        let px = &mut leds[self.port];
        if self.is_dead {
            px.set_clamped(self.style.r, self.style.g, self.style.b);
            return;
        }
        let v = utils::invert(self.faker.as_mut().map(|f| f.trig(now)).unwrap_or(0.0));
        px.set_clamped(v * self.style.r, v * self.style.g, v * self.style.b);
    }
}

impl BlinkStyle {
    /// Build a [`PortState`] for `port` (mirrors `BlinkStyle.CreatePort`):
    /// dead-port and slow/fast decisions via the context RNG. Timing is shaped
    /// by the controller's virtual traffic clock.
    pub fn create_port(&self, port: usize, ctx: &mut BootstrapCtx) -> PortState {
        if ctx.rng.range_f32(0.0, 1.0) < self.dead_port_chance {
            return PortState {
                faker: None,
                port,
                style: self.dead_color,
                is_dead: true,
            };
        }

        let style = if ctx.rng.range_f32(0.0, 1.0) < self.slow_speed_chance {
            self.slow_color
        } else {
            self.fast_color
        };

        let blinker = RandomBlinker::new(
            self.min_blinks,
            self.max_blinks,
            self.min_cycle,
            self.max_cycle,
            0,
            Rng::new(ctx.rng.next_seed()),
        );
        let faker = RandomInterval::new(
            self.min_interval,
            self.max_interval,
            self.min_blink,
            self.max_blink,
            Box::new(blinker),
            0,
            Rng::new(ctx.rng.next_seed()),
        );

        PortState {
            faker: Some(Box::new(faker)),
            port,
            style,
            is_dead: false,
        }
    }
}

// Keep the original public helper API while the hardware-specific definitions
// live with their respective chassis families.
pub const CISCO7609_DEAD_PORT_CHANCE: f32 = super::cisco7609::blinkstyle::DEAD_PORT_CHANCE;
pub const CISCO7609_HEALTHY_COLOR: ColorStyle = super::cisco7609::blinkstyle::HEALTHY_COLOR;
pub const CISCO7609_100MBIT_COLOR: ColorStyle = super::cisco7609::blinkstyle::COLOR_100MBIT;
pub const CISCO7609_100MBIT_PORT_CHANCE: f32 = super::cisco7609::blinkstyle::PORT_100MBIT_CHANCE;

pub fn asr9000_style() -> BlinkStyle {
    super::asr9000::blinkstyle::asr9000_style()
}

pub fn cisco7609_style() -> BlinkStyle {
    super::cisco7609::blinkstyle::cisco7609_style()
}
