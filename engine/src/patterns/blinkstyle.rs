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

/// Cisco 7609 link LEDs use solid red to mean a down/dead port. Healthy ports
/// are green for gigabit or orange for 100 Mbit/s.
pub const CISCO7609_DEAD_PORT_CHANCE: f32 = 0.03;
pub const CISCO7609_HEALTHY_COLOR: ColorStyle = ColorStyle {
    r: 0.0,
    g: 1.0,
    b: 0.0,
};
pub const CISCO7609_100MBIT_COLOR: ColorStyle = ColorStyle {
    r: 1.0,
    // This is perceptual sRGB and is linearized before APA102 output. A high
    // green component keeps the result visibly orange rather than red.
    g: 0.8,
    b: 0.0,
};
pub const CISCO7609_100MBIT_PORT_CHANCE: f32 = 0.05;

// `slow_speed_chance` is sampled only after a port survives the dead-port
// roll, so compensate to keep the unconditional orange population at 5%.
const CISCO7609_100MBIT_SURVIVOR_CHANCE: f32 =
    CISCO7609_100MBIT_PORT_CHANCE / (1.0 - CISCO7609_DEAD_PORT_CHANCE);

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
    /// dead-port and slow/fast decisions via the ctx RNG, timings scaled by the
    /// ctx intensity.
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

        let inv = 1.0 / ctx.intensity.max(0.1); // scaled_interval divides by intensity
        let blinker = RandomBlinker::new(
            self.min_blinks,
            self.max_blinks,
            self.min_cycle,
            self.max_cycle,
            0,
            Rng::new(ctx.rng.next_seed()),
        );
        let faker = RandomInterval::new(
            self.min_interval * inv,
            self.max_interval * inv,
            self.min_blink * inv,
            self.max_blink * inv,
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

/// Blink style for ASR9000 40GE linecards (mirrors `ASR9000Style`).
pub fn asr9000_style() -> BlinkStyle {
    BlinkStyle {
        min_interval: 0.05,
        max_interval: 7.0,
        min_blink: 0.05,
        max_blink: 12.0,
        min_blinks: 15.0,
        max_blinks: 40.0,
        min_cycle: 1.0,
        max_cycle: 10.0,
        slow_color: ColorStyle {
            r: 1.0,
            g: 0.6,
            b: 0.0,
        },
        fast_color: ColorStyle {
            r: 0.3,
            g: 1.0,
            b: 0.0,
        },
        dead_color: ColorStyle {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        dead_port_chance: 0.067,
        slow_speed_chance: 0.2,
    }
}

/// Blink style for Cisco 7609 linecards. Unlike the last Go default, the
/// physical installation reserves solid red for an approximately 3% set of
/// down/dead ports.
pub fn cisco7609_style() -> BlinkStyle {
    BlinkStyle {
        min_interval: 0.1,
        max_interval: 7.0,
        min_blink: 0.1,
        max_blink: 12.0,
        min_blinks: 15.0,
        max_blinks: 40.0,
        min_cycle: 1.0,
        max_cycle: 10.0,
        slow_color: CISCO7609_100MBIT_COLOR,
        fast_color: CISCO7609_HEALTHY_COLOR,
        dead_color: ColorStyle {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        dead_port_chance: CISCO7609_DEAD_PORT_CHANCE,
        slow_speed_chance: CISCO7609_100MBIT_SURVIVOR_CHANCE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faker::Rng;

    #[test]
    fn cisco_port_populations_and_colors_match_link_semantics() {
        let style = cisco7609_style();
        assert_eq!(style.dead_port_chance, 0.03);
        assert!(((1.0 - style.dead_port_chance) * style.slow_speed_chance - 0.05).abs() < 1e-6);
        assert_eq!(style.slow_color.r, 1.0);
        assert_eq!(style.slow_color.g, 0.8);
        assert_eq!(style.slow_color.b, 0.0);
        assert_eq!(style.fast_color.r, 0.0);
        assert_eq!(style.fast_color.g, 1.0);
        assert_eq!(style.fast_color.b, 0.0);

        let mut always_100mbit = style;
        always_100mbit.dead_port_chance = 0.0;
        always_100mbit.slow_speed_chance = 1.0;
        let mut rng = Rng::new(7);
        let mut ctx = BootstrapCtx {
            rng: &mut rng,
            intensity: 1.0,
        };
        let port = always_100mbit.create_port(0, &mut ctx);
        assert!(!port.is_dead);
        assert!(port.faker.is_some());
        assert_eq!(port.style.r, 1.0);
        assert_eq!(port.style.g, 0.8);
        assert_eq!(port.style.b, 0.0);
    }

    #[test]
    fn cisco_dead_ports_are_solid_red_without_a_faker() {
        let style = cisco7609_style();

        let mut always_dead = style;
        always_dead.dead_port_chance = 1.0;
        let mut rng = Rng::new(7);
        let mut ctx = BootstrapCtx {
            rng: &mut rng,
            intensity: 1.0,
        };
        let mut port = always_dead.create_port(0, &mut ctx);
        let mut leds = [Pixel::new()];

        port.render(&mut leds, 0);
        assert_eq!(leds[0].to_srgb8(), [255, 0, 0]);
        port.render(&mut leds, 60_000);
        assert_eq!(leds[0].to_srgb8(), [255, 0, 0]);
    }

    #[test]
    fn asr_dead_port_probability_is_unchanged() {
        assert_eq!(asr9000_style().dead_port_chance, 0.067);
    }
}
