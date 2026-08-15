//! Shared, real-time link-port traffic model.
//!
//! Link state remains visible as a steady base color. Traffic is represented
//! by short, irregular bursts whose arrival rate follows the controller's
//! effective traffic intensity; the visible flicker cadence stays tied to
//! wall time and is therefore not stretched by NTP shaping or POST ramping.

use crate::faker::Rng;
use crate::pattern::{BootstrapCtx, LinkActivation, RenderInfo};
use crate::pixel::Pixel;

/// RGB multipliers in `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorStyle {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// Half-open millisecond range used for deterministic timing samples.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MillisRange {
    pub min: u32,
    pub max: u32,
}

impl MillisRange {
    pub const fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }

    fn sample(self, rng: &mut Rng) -> u32 {
        if self.max <= self.min {
            self.min
        } else {
            rng.range_f32(self.min as f32, self.max as f32) as u32
        }
    }
}

/// Persistent activity class assigned to a port at bootstrap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrafficRole {
    Light,
    Medium,
    Heavy,
}

impl TrafficRole {
    const fn index(self) -> usize {
        match self {
            Self::Light => 0,
            Self::Medium => 1,
            Self::Heavy => 2,
        }
    }
}

const ROLE_IDLE: [MillisRange; 3] = [
    MillisRange::new(1_500, 5_001),
    MillisRange::new(400, 1_801),
    MillisRange::new(80, 451),
];
const ROLE_BURST: [MillisRange; 3] = [
    MillisRange::new(250, 901),
    MillisRange::new(500, 1_801),
    MillisRange::new(1_000, 3_501),
];

/// Typed traffic tuning for one card family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityProfile {
    /// Light, medium, and heavy probability weights. They must total 1.0.
    pub role_weights: [f32; 3],
    pub bright_ms: MillisRange,
    pub dim_ms: MillisRange,
    pub dim_factor: [f32; 2],
    pub initial_hold_ms: MillisRange,
}

impl ActivityProfile {
    pub const fn new(
        role_weights: [f32; 3],
        bright_ms: MillisRange,
        dim_ms: MillisRange,
        dim_factor: [f32; 2],
    ) -> Self {
        Self {
            role_weights,
            bright_ms,
            dim_ms,
            dim_factor,
            initial_hold_ms: MillisRange::new(100, 501),
        }
    }

    fn choose_role(self, rng: &mut Rng) -> TrafficRole {
        let sample = rng.range_f32(0.0, 1.0);
        if sample < self.role_weights[0] {
            TrafficRole::Light
        } else if sample < self.role_weights[0] + self.role_weights[1] {
            TrafficRole::Medium
        } else {
            TrafficRole::Heavy
        }
    }
}

/// How an active traffic pulse differs from the steady link indication.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActivityEffect {
    Dim,
    Alternate(ColorStyle),
}

/// Colors, populations, and traffic tuning for a set of ports.
#[derive(Clone, Copy, Debug)]
pub struct BlinkStyle {
    pub slow_color: ColorStyle,
    pub fast_color: ColorStyle,
    pub dead_color: ColorStyle,
    pub dead_port_chance: f32,
    pub slow_speed_chance: f32,
    pub activity: ActivityProfile,
    pub effect: ActivityEffect,
}

#[derive(Clone, Copy, Debug)]
enum ActivityPhase {
    AwaitingActivation,
    Holding {
        until_ms: u32,
    },
    Idle {
        work_remaining_ms: f32,
    },
    Burst {
        until_ms: u32,
        next_toggle_ms: u32,
        dimmed: bool,
        dim_factor: f32,
    },
    Steady,
}

/// One port's persistent identity and runtime traffic state.
pub struct PortState {
    port: usize,
    base_color: ColorStyle,
    dead_color: ColorStyle,
    is_dead: bool,
    role: TrafficRole,
    profile: ActivityProfile,
    effect: ActivityEffect,
    rng: Rng,
    activation_at_ms: Option<u32>,
    phase: ActivityPhase,
    last_now_ms: Option<u32>,
}

impl PortState {
    /// Render a link using real elapsed time and the current traffic intensity.
    pub fn render(&mut self, leds: &mut [Pixel], info: &RenderInfo) {
        let now_ms = info.millis;
        let activation_at = *self.activation_at_ms.get_or_insert(now_ms);
        if !deadline_reached(now_ms, activation_at) {
            self.set_black(leds);
            self.last_now_ms = Some(now_ms);
            return;
        }

        if matches!(self.phase, ActivityPhase::AwaitingActivation) {
            if self.is_dead {
                self.phase = ActivityPhase::Steady;
            } else {
                self.phase = ActivityPhase::Holding {
                    until_ms: now_ms
                        .wrapping_add(self.profile.initial_hold_ms.sample(&mut self.rng)),
                };
            }
            self.last_now_ms = Some(now_ms);
        }

        let previous_ms = self.last_now_ms.replace(now_ms).unwrap_or(now_ms);
        match self.phase {
            ActivityPhase::AwaitingActivation => self.set_black(leds),
            ActivityPhase::Steady => self.set_color(leds, self.dead_color),
            ActivityPhase::Holding { until_ms } => {
                if deadline_reached(now_ms, until_ms) && info.traffic_intensity > 0.0 {
                    self.start_burst(now_ms);
                    self.render_burst(leds);
                } else {
                    self.set_color(leds, self.base_color);
                }
            }
            ActivityPhase::Idle {
                mut work_remaining_ms,
            } => {
                let elapsed_ms = now_ms.wrapping_sub(previous_ms) as f32;
                work_remaining_ms -= elapsed_ms * info.traffic_intensity.max(0.0);
                if work_remaining_ms <= 0.0 {
                    self.start_burst(now_ms);
                    self.render_burst(leds);
                } else {
                    self.phase = ActivityPhase::Idle { work_remaining_ms };
                    self.set_color(leds, self.base_color);
                }
            }
            ActivityPhase::Burst {
                until_ms,
                mut next_toggle_ms,
                mut dimmed,
                mut dim_factor,
            } => {
                if deadline_reached(now_ms, until_ms) {
                    self.start_idle();
                    self.set_color(leds, self.base_color);
                    return;
                }
                if deadline_reached(now_ms, next_toggle_ms) {
                    dimmed = !dimmed;
                    let range = if dimmed {
                        dim_factor = self
                            .rng
                            .range_f32(self.profile.dim_factor[0], self.profile.dim_factor[1]);
                        self.profile.dim_ms
                    } else {
                        self.profile.bright_ms
                    };
                    next_toggle_ms = now_ms.wrapping_add(range.sample(&mut self.rng));
                }
                self.phase = ActivityPhase::Burst {
                    until_ms,
                    next_toggle_ms,
                    dimmed,
                    dim_factor,
                };
                self.render_burst(leds);
            }
        }
    }

    /// Apply a new link-up schedule while preserving color and traffic role.
    pub fn restart_link_traffic(&mut self, now_ms: u32, activations: &[LinkActivation]) {
        let delay_ms = activations
            .iter()
            .find(|activation| activation.led == self.port)
            .map_or(0, |activation| activation.delay_ms);
        self.activation_at_ms = Some(now_ms.wrapping_add(delay_ms));
        self.phase = ActivityPhase::AwaitingActivation;
        self.last_now_ms = Some(now_ms);
    }

    fn start_burst(&mut self, now_ms: u32) {
        let duration = ROLE_BURST[self.role.index()].sample(&mut self.rng);
        let dim_duration = self.profile.dim_ms.sample(&mut self.rng);
        let dim_factor = self
            .rng
            .range_f32(self.profile.dim_factor[0], self.profile.dim_factor[1]);
        self.phase = ActivityPhase::Burst {
            until_ms: now_ms.wrapping_add(duration),
            next_toggle_ms: now_ms.wrapping_add(dim_duration),
            dimmed: true,
            dim_factor,
        };
    }

    fn start_idle(&mut self) {
        self.phase = ActivityPhase::Idle {
            work_remaining_ms: ROLE_IDLE[self.role.index()].sample(&mut self.rng) as f32,
        };
    }

    fn render_burst(&self, leds: &mut [Pixel]) {
        let ActivityPhase::Burst {
            dimmed, dim_factor, ..
        } = self.phase
        else {
            return;
        };
        if !dimmed {
            self.set_color(leds, self.base_color);
            return;
        }
        match self.effect {
            ActivityEffect::Dim => self.set_color(
                leds,
                ColorStyle {
                    r: self.base_color.r * dim_factor,
                    g: self.base_color.g * dim_factor,
                    b: self.base_color.b * dim_factor,
                },
            ),
            ActivityEffect::Alternate(color) => self.set_color(leds, color),
        }
    }

    fn set_black(&self, leds: &mut [Pixel]) {
        self.set_color(
            leds,
            ColorStyle {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
        );
    }

    fn set_color(&self, leds: &mut [Pixel], color: ColorStyle) {
        if let Some(pixel) = leds.get_mut(self.port) {
            pixel.set_clamped(color.r, color.g, color.b);
        }
    }
}

impl BlinkStyle {
    /// Build a port with stable color/failure identity and traffic role.
    pub fn create_port(&self, port: usize, ctx: &mut BootstrapCtx) -> PortState {
        let is_dead = ctx.rng.range_f32(0.0, 1.0) < self.dead_port_chance;
        let base_color = if ctx.rng.range_f32(0.0, 1.0) < self.slow_speed_chance {
            self.slow_color
        } else {
            self.fast_color
        };
        let role = self.activity.choose_role(ctx.rng);
        PortState {
            port,
            base_color,
            dead_color: self.dead_color,
            is_dead,
            role,
            profile: self.activity,
            effect: self.effect,
            rng: Rng::new(ctx.rng.next_seed()),
            activation_at_ms: None,
            phase: ActivityPhase::AwaitingActivation,
            last_now_ms: None,
        }
    }
}

fn deadline_reached(now: u32, deadline: u32) -> bool {
    now.wrapping_sub(deadline) < (1 << 31)
}

// Keep the original public helper API while hardware-specific definitions live
// with their respective chassis families.
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

#[cfg(test)]
mod tests {
    use super::*;

    const GREEN: ColorStyle = ColorStyle {
        r: 0.0,
        g: 1.0,
        b: 0.0,
    };
    const RED: ColorStyle = ColorStyle {
        r: 1.0,
        g: 0.0,
        b: 0.0,
    };
    const PROFILE: ActivityProfile = ActivityProfile {
        role_weights: [1.0, 0.0, 0.0],
        bright_ms: MillisRange::new(70, 71),
        dim_ms: MillisRange::new(35, 36),
        dim_factor: [0.25, 0.250_001],
        initial_hold_ms: MillisRange::new(100, 101),
    };

    fn healthy_port() -> PortState {
        let style = BlinkStyle {
            slow_color: GREEN,
            fast_color: GREEN,
            dead_color: RED,
            dead_port_chance: 0.0,
            slow_speed_chance: 0.0,
            activity: PROFILE,
            effect: ActivityEffect::Dim,
        };
        let mut rng = Rng::new(7);
        style.create_port(0, &mut BootstrapCtx { rng: &mut rng })
    }

    fn info(millis: u32, traffic_intensity: f32) -> RenderInfo {
        RenderInfo {
            millis,
            traffic_intensity,
            ..RenderInfo::default()
        }
    }

    #[test]
    fn negotiated_port_lights_before_its_first_flicker() {
        let mut port = healthy_port();
        let mut leds = [Pixel::new()];
        port.restart_link_traffic(
            1_000,
            &[LinkActivation {
                led: 0,
                delay_ms: 500,
            }],
        );

        port.render(&mut leds, &info(1_499, 1.0));
        assert_eq!(leds[0].to_srgb8(), [0, 0, 0]);
        port.render(&mut leds, &info(1_500, 1.0));
        assert_eq!(leds[0].to_srgb8(), [0, 255, 0]);
        port.render(&mut leds, &info(1_599, 1.0));
        assert_eq!(leds[0].to_srgb8(), [0, 255, 0]);
        port.render(&mut leds, &info(1_600, 1.0));
        assert_eq!(leds[0].to_srgb8(), [0, 63, 0]);
    }

    #[test]
    fn zero_intensity_holds_a_link_steady() {
        let mut port = healthy_port();
        let mut leds = [Pixel::new()];
        port.render(&mut leds, &info(0, 0.0));
        port.render(&mut leds, &info(100, 0.0));
        port.render(&mut leds, &info(1_001, 0.0));
        assert_eq!(leds[0].to_srgb8(), [0, 255, 0]);
        port.render(&mut leds, &info(60_000, 0.0));
        assert_eq!(leds[0].to_srgb8(), [0, 255, 0]);
    }

    #[test]
    fn higher_intensity_produces_more_activity_without_changing_cadence() {
        let mut low = healthy_port();
        let mut high = healthy_port();
        let mut low_leds = [Pixel::new()];
        let mut high_leds = [Pixel::new()];
        let mut low_dim_frames = 0;
        let mut high_dim_frames = 0;

        for millis in (0..120_000).step_by(10) {
            low.render(&mut low_leds, &info(millis, 0.2));
            high.render(&mut high_leds, &info(millis, 1.0));
            low_dim_frames += usize::from(low_leds[0].to_srgb8()[1] < 255);
            high_dim_frames += usize::from(high_leds[0].to_srgb8()[1] < 255);
        }

        assert!(high_dim_frames > low_dim_frames * 2);
        assert_eq!(low.profile.bright_ms, high.profile.bright_ms);
        assert_eq!(low.profile.dim_ms, high.profile.dim_ms);
    }

    #[test]
    fn seeded_role_population_tracks_profile_weights() {
        let style = BlinkStyle {
            slow_color: GREEN,
            fast_color: GREEN,
            dead_color: RED,
            dead_port_chance: 0.0,
            slow_speed_chance: 0.0,
            activity: ActivityProfile::new(
                [0.55, 0.35, 0.10],
                MillisRange::new(70, 161),
                MillisRange::new(35, 81),
                [0.20, 0.45],
            ),
            effect: ActivityEffect::Dim,
        };
        let mut rng = Rng::new(42);
        let mut counts = [0_usize; 3];
        for port_index in 0..10_000 {
            let port = style.create_port(port_index, &mut BootstrapCtx { rng: &mut rng });
            counts[port.role.index()] += 1;
        }

        assert!((counts[0] as i32 - 5_500).abs() < 250);
        assert!((counts[1] as i32 - 3_500).abs() < 250);
        assert!((counts[2] as i32 - 1_000).abs() < 150);
    }

    #[test]
    fn restart_preserves_port_identity_and_role() {
        let mut port = healthy_port();
        let role = port.role;
        let color = port.base_color;
        port.restart_link_traffic(10_000, &[]);
        assert_eq!(port.role, role);
        assert_eq!(port.base_color, color);
    }

    #[test]
    fn deadlines_survive_u32_wrap() {
        let mut port = healthy_port();
        let mut leds = [Pixel::new()];
        let start = u32::MAX - 50;
        port.restart_link_traffic(
            start,
            &[LinkActivation {
                led: 0,
                delay_ms: 100,
            }],
        );
        port.render(&mut leds, &info(start.wrapping_add(99), 1.0));
        assert_eq!(leds[0].to_srgb8(), [0, 0, 0]);
        port.render(&mut leds, &info(start.wrapping_add(100), 1.0));
        assert_eq!(leds[0].to_srgb8(), [0, 255, 0]);
    }
}
