//! Platform-neutral redundant-power state tracking and final-frame overrides.

use alloc::vec::Vec;

use crate::faker::Rng;
use crate::pixel::Pixel;

pub const POST_AMBER_MS: u64 = 10_000;
pub const POST_SWEEP_STEP_MS: u64 = 90;
pub const POST_BLACK_MS: u64 = 8_000;
pub const POST_TRAFFIC_RAMP_MS: u64 = 30_000;
pub const POST_NEGOTIATION_MAX_MS: u64 = 5_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedundantPowerState {
    Healthy,
    Degraded,
    Offline,
}

impl RedundantPowerState {
    pub const fn from_online(online: [bool; 2]) -> Self {
        match online {
            [true, true] => Self::Healthy,
            [false, false] => Self::Offline,
            _ => Self::Degraded,
        }
    }

    /// Apply the PSU indication after a normal internal render. Offline uses
    /// the same terminal all-black frame as the blackout pattern, without
    /// replacing the active scene underneath it.
    pub fn apply(self, leds: &mut [Pixel], mgmt_led: usize) {
        if self == Self::Offline {
            for pixel in leds {
                pixel.set_clamped(0.0, 0.0, 0.0);
            }
            return;
        }

        let Some(mgmt) = leds.get_mut(mgmt_led) else {
            return;
        };
        match self {
            Self::Healthy => mgmt.set_clamped(0.2, 1.0, 0.0),
            Self::Degraded => mgmt.set_clamped(1.0, 0.0, 0.0),
            Self::Offline => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ActiveLowDebouncer {
    debounce_ms: u64,
    sampled_low: Option<bool>,
    stable_low: Option<bool>,
    sample_changed_ms: u64,
}

impl ActiveLowDebouncer {
    const fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            sampled_low: None,
            stable_low: None,
            sample_changed_ms: 0,
        }
    }

    fn update(&mut self, low: bool, now_ms: u64) -> bool {
        let Some(sampled_low) = self.sampled_low else {
            self.sampled_low = Some(low);
            self.sample_changed_ms = now_ms;
            return false;
        };
        if low != sampled_low {
            self.sampled_low = Some(low);
            self.sample_changed_ms = now_ms;
        }
        if self.sampled_low == self.stable_low
            || now_ms.saturating_sub(self.sample_changed_ms) < self.debounce_ms
        {
            return false;
        }

        self.stable_low = self.sampled_low;
        true
    }
}

/// Debounces two inputs and emits only combined PSU-state transitions. It
/// starts with both inputs offline so boot and disconnected pins fail safely.
#[derive(Clone, Copy, Debug)]
pub struct RedundantPowerMonitor {
    inputs: [ActiveLowDebouncer; 2],
    state: RedundantPowerState,
    initialized: bool,
}

impl RedundantPowerMonitor {
    pub const fn new(debounce_ms: u64) -> Self {
        Self {
            inputs: [ActiveLowDebouncer::new(debounce_ms); 2],
            state: RedundantPowerState::Offline,
            initialized: false,
        }
    }

    pub const fn state(&self) -> RedundantPowerState {
        self.state
    }

    pub const fn initialized(&self) -> bool {
        self.initialized
    }

    /// `low` is the raw electrical input level for each pin. Low means online.
    pub fn update(&mut self, low: [bool; 2], now_ms: u64) -> Option<RedundantPowerState> {
        let changed = self
            .inputs
            .iter_mut()
            .zip(low)
            .fold(false, |changed, (input, low)| {
                input.update(low, now_ms) || changed
            });
        if !changed || self.inputs.iter().any(|input| input.stable_low.is_none()) {
            return None;
        }

        let next = RedundantPowerState::from_online([
            self.inputs[0].stable_low.unwrap_or(false),
            self.inputs[1].stable_low.unwrap_or(false),
        ]);
        if self.initialized && next == self.state {
            return None;
        }
        self.state = next;
        self.initialized = true;
        Some(next)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PowerSequencePhase {
    Initializing,
    Offline,
    Normal,
    Amber,
    Sweep,
    Black,
    Ramp,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PowerSequenceStatus {
    pub phase: PowerSequencePhase,
    pub traffic_scale: f32,
    pub restart_traffic: bool,
    sweep_off_pixels: usize,
    negotiation_elapsed_ms: u64,
}

#[derive(Clone, Copy, Debug)]
enum SequenceMode {
    Initializing,
    Offline,
    Normal,
    Post { started_ms: u64 },
}

/// Power-on lifecycle armed by a confirmed both-off PSU state. The sequence is
/// intentionally independent of DDP; callers decide whether its physical
/// frame override is applied.
pub struct PowerOnSequence {
    sweep_order: Vec<usize>,
    negotiation_leds: Vec<usize>,
    negotiation_delays_ms: Vec<u64>,
    rng: Rng,
    mode: SequenceMode,
    restart_emitted: bool,
}

impl PowerOnSequence {
    pub fn new(sweep_order: Vec<usize>, negotiation_leds: Vec<usize>, seed: u64) -> Self {
        let negotiation_delays_ms = alloc::vec![0; negotiation_leds.len()];
        Self {
            sweep_order,
            negotiation_leds,
            negotiation_delays_ms,
            rng: Rng::new(seed),
            mode: SequenceMode::Initializing,
            restart_emitted: false,
        }
    }

    /// Feed only debounced state observations emitted by
    /// [`RedundantPowerMonitor`], including its first initialized state.
    pub fn observe_power(&mut self, state: RedundantPowerState, now_ms: u64) {
        if state == RedundantPowerState::Offline {
            self.mode = SequenceMode::Offline;
            self.restart_emitted = false;
            return;
        }

        match self.mode {
            SequenceMode::Initializing => self.mode = SequenceMode::Normal,
            SequenceMode::Offline => {
                for delay in &mut self.negotiation_delays_ms {
                    *delay = self.rng.range_f32(0.0, POST_NEGOTIATION_MAX_MS as f32) as u64;
                }
                self.mode = SequenceMode::Post { started_ms: now_ms };
                self.restart_emitted = false;
            }
            SequenceMode::Normal | SequenceMode::Post { .. } => {}
        }
    }

    pub fn update(&mut self, now_ms: u64) -> PowerSequenceStatus {
        let mut restart_traffic = false;
        let (phase, traffic_scale, sweep_off_pixels, negotiation_elapsed_ms, complete) = match self
            .mode
        {
            SequenceMode::Initializing => (PowerSequencePhase::Initializing, 0.0, 0, 0, false),
            SequenceMode::Offline => (PowerSequencePhase::Offline, 0.0, 0, 0, false),
            SequenceMode::Normal => (PowerSequencePhase::Normal, 1.0, 0, u64::MAX, false),
            SequenceMode::Post { started_ms } => {
                let elapsed = now_ms.saturating_sub(started_ms);
                let sweep_ms = self.sweep_order.len() as u64 * POST_SWEEP_STEP_MS;
                let black_start = POST_AMBER_MS + sweep_ms;
                let ramp_start = black_start + POST_BLACK_MS;
                let complete = ramp_start + POST_TRAFFIC_RAMP_MS;

                if elapsed < POST_AMBER_MS {
                    (PowerSequencePhase::Amber, 0.0, 0, 0, false)
                } else if elapsed < black_start {
                    let off = ((elapsed - POST_AMBER_MS) / POST_SWEEP_STEP_MS + 1)
                        .min(self.sweep_order.len() as u64) as usize;
                    (PowerSequencePhase::Sweep, 0.0, off, 0, false)
                } else if elapsed < ramp_start {
                    (
                        PowerSequencePhase::Black,
                        0.0,
                        self.sweep_order.len(),
                        0,
                        false,
                    )
                } else if elapsed < complete {
                    let progress = (elapsed - ramp_start) as f32 / POST_TRAFFIC_RAMP_MS as f32;
                    (
                        PowerSequencePhase::Ramp,
                        progress,
                        self.sweep_order.len(),
                        elapsed - ramp_start,
                        false,
                    )
                } else {
                    (
                        PowerSequencePhase::Normal,
                        1.0,
                        self.sweep_order.len(),
                        u64::MAX,
                        true,
                    )
                }
            }
        };

        if matches!(self.mode, SequenceMode::Post { started_ms } if now_ms.saturating_sub(started_ms)
            >= POST_AMBER_MS + self.sweep_order.len() as u64 * POST_SWEEP_STEP_MS + POST_BLACK_MS)
            && !self.restart_emitted
        {
            self.restart_emitted = true;
            restart_traffic = true;
        }
        if complete {
            self.mode = SequenceMode::Normal;
        }

        PowerSequenceStatus {
            phase,
            traffic_scale,
            restart_traffic,
            sweep_off_pixels,
            negotiation_elapsed_ms,
        }
    }

    /// Apply physical output ownership. Ramp leaves status/panel LEDs visible
    /// immediately while holding each link LED black for its negotiation delay.
    pub fn apply_physical(&self, status: PowerSequenceStatus, leds: &mut [Pixel]) {
        match status.phase {
            PowerSequencePhase::Initializing
            | PowerSequencePhase::Offline
            | PowerSequencePhase::Black => fill(leds, 0.0, 0.0, 0.0),
            PowerSequencePhase::Amber => fill(leds, 1.0, 0.8, 0.0),
            PowerSequencePhase::Sweep => {
                fill(leds, 1.0, 0.8, 0.0);
                for &index in self.sweep_order.iter().take(status.sweep_off_pixels) {
                    if let Some(pixel) = leds.get_mut(index) {
                        pixel.set_clamped(0.0, 0.0, 0.0);
                    }
                }
            }
            PowerSequencePhase::Ramp => {
                for (&index, &delay_ms) in self
                    .negotiation_leds
                    .iter()
                    .zip(&self.negotiation_delays_ms)
                {
                    if status.negotiation_elapsed_ms < delay_ms
                        && let Some(pixel) = leds.get_mut(index)
                    {
                        pixel.set_clamped(0.0, 0.0, 0.0);
                    }
                }
            }
            PowerSequencePhase::Normal => {}
        }
    }
}

fn fill(leds: &mut [Pixel], r: f32, g: f32, b: f32) {
    for pixel in leds {
        pixel.set_clamped(r, g, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit_leds() -> [Pixel; 3] {
        let mut leds = [Pixel::new(); 3];
        for pixel in &mut leds {
            pixel.set_clamped(0.4, 0.5, 0.6);
        }
        leds
    }

    #[test]
    fn derives_all_redundant_power_states() {
        assert_eq!(
            RedundantPowerState::from_online([true, true]),
            RedundantPowerState::Healthy
        );
        assert_eq!(
            RedundantPowerState::from_online([true, false]),
            RedundantPowerState::Degraded
        );
        assert_eq!(
            RedundantPowerState::from_online([false, true]),
            RedundantPowerState::Degraded
        );
        assert_eq!(
            RedundantPowerState::from_online([false, false]),
            RedundantPowerState::Offline
        );
    }

    #[test]
    fn debounces_combined_transitions_and_starts_offline() {
        let mut monitor = RedundantPowerMonitor::new(30);
        assert_eq!(monitor.state(), RedundantPowerState::Offline);
        assert_eq!(monitor.update([true, true], 0), None);
        assert_eq!(monitor.update([true, true], 29), None);
        assert_eq!(
            monitor.update([true, true], 30),
            Some(RedundantPowerState::Healthy)
        );

        assert_eq!(monitor.update([false, true], 40), None);
        assert_eq!(monitor.update([true, true], 50), None);
        assert_eq!(monitor.update([false, true], 60), None);
        assert_eq!(
            monitor.update([false, true], 90),
            Some(RedundantPowerState::Degraded)
        );
        assert_eq!(monitor.update([false, false], 120), None,);
        assert_eq!(
            monitor.update([false, false], 150),
            Some(RedundantPowerState::Offline)
        );
    }

    #[test]
    fn emits_a_confirmed_initial_offline_observation() {
        let mut monitor = RedundantPowerMonitor::new(30);
        assert_eq!(monitor.update([false, false], 100), None);
        assert_eq!(monitor.update([false, false], 129), None);
        assert_eq!(
            monitor.update([false, false], 130),
            Some(RedundantPowerState::Offline)
        );
        assert!(monitor.initialized());
    }

    #[test]
    fn applies_green_red_and_black_output_overrides() {
        let mut leds = lit_leds();
        RedundantPowerState::Healthy.apply(&mut leds, 1);
        assert_eq!(leds[0].to_srgb8(), [102, 127, 153]);
        assert_eq!(leds[1].to_srgb8(), [51, 255, 0]);

        RedundantPowerState::Degraded.apply(&mut leds, 1);
        assert_eq!(leds[1].to_srgb8(), [255, 0, 0]);

        RedundantPowerState::Offline.apply(&mut leds, 1);
        assert!(leds.iter().all(|pixel| pixel.to_srgb8() == [0, 0, 0]));
    }

    #[test]
    fn online_at_initialization_skips_post_but_both_off_arms_it() {
        let mut sequence = PowerOnSequence::new((0..3).collect(), vec![0, 1, 2], 1);
        sequence.observe_power(RedundantPowerState::Healthy, 30);
        assert_eq!(sequence.update(30).phase, PowerSequencePhase::Normal);

        sequence.observe_power(RedundantPowerState::Degraded, 100);
        assert_eq!(sequence.update(100).phase, PowerSequencePhase::Normal);
        sequence.observe_power(RedundantPowerState::Offline, 200);
        assert_eq!(sequence.update(200).phase, PowerSequencePhase::Offline);
        sequence.observe_power(RedundantPowerState::Degraded, 300);
        assert_eq!(sequence.update(300).phase, PowerSequencePhase::Amber);
        assert!(
            sequence
                .negotiation_delays_ms
                .iter()
                .all(|delay| *delay < POST_NEGOTIATION_MAX_MS)
        );
        assert!(
            sequence
                .negotiation_delays_ms
                .iter()
                .any(|delay| *delay > 0)
        );
        sequence.observe_power(RedundantPowerState::Healthy, 400);
        assert_eq!(sequence.update(400).phase, PowerSequencePhase::Amber);
    }

    #[test]
    fn post_timing_sweep_and_ramp_match_the_chassis_sequence() {
        let mut sequence = PowerOnSequence::new(vec![2, 0, 1], vec![], 1);
        sequence.observe_power(RedundantPowerState::Offline, 0);
        sequence.observe_power(RedundantPowerState::Healthy, 1_000);

        let mut leds = lit_leds();
        let amber_end = 1_000 + POST_AMBER_MS;
        let status = sequence.update(amber_end - 1);
        sequence.apply_physical(status, &mut leds);
        assert_eq!(status.phase, PowerSequencePhase::Amber);
        assert!(leds.iter().all(|pixel| pixel.to_srgb8() == [255, 204, 0]));

        let status = sequence.update(amber_end);
        sequence.apply_physical(status, &mut leds);
        assert_eq!(status.phase, PowerSequencePhase::Sweep);
        assert_eq!(leds[2].to_srgb8(), [0, 0, 0]);
        assert_eq!(leds[0].to_srgb8(), [255, 204, 0]);

        let black_start = amber_end + 3 * POST_SWEEP_STEP_MS;
        assert_eq!(
            sequence.update(black_start).phase,
            PowerSequencePhase::Black
        );
        let ramp_start = black_start + POST_BLACK_MS;
        let status = sequence.update(ramp_start);
        assert_eq!(status.phase, PowerSequencePhase::Ramp);
        assert_eq!(status.traffic_scale, 0.0);
        assert!(status.restart_traffic);
        assert!(!sequence.update(ramp_start + 1).restart_traffic);
        let halfway = sequence.update(ramp_start + POST_TRAFFIC_RAMP_MS / 2);
        assert!((halfway.traffic_scale - 0.5).abs() < 1e-6);
        assert_eq!(
            sequence.update(ramp_start + POST_TRAFFIC_RAMP_MS).phase,
            PowerSequencePhase::Normal
        );
    }

    #[test]
    fn returning_offline_cancels_post_and_rearms() {
        let mut sequence = PowerOnSequence::new(vec![0], vec![], 1);
        sequence.observe_power(RedundantPowerState::Offline, 0);
        sequence.observe_power(RedundantPowerState::Degraded, 100);
        assert_eq!(sequence.update(100).phase, PowerSequencePhase::Amber);
        sequence.observe_power(RedundantPowerState::Offline, 200);
        assert_eq!(sequence.update(200).phase, PowerSequencePhase::Offline);
        sequence.observe_power(RedundantPowerState::Healthy, 300);
        assert_eq!(sequence.update(300).phase, PowerSequencePhase::Amber);
    }

    #[test]
    fn ramp_reveals_only_link_leds_after_their_negotiation_delays() {
        let mut sequence = PowerOnSequence::new(vec![0, 1, 2], vec![0, 2], 7);
        sequence.negotiation_delays_ms = vec![1_000, 4_000];
        sequence.mode = SequenceMode::Post { started_ms: 0 };
        sequence.restart_emitted = true;
        let ramp_start = POST_AMBER_MS + 3 * POST_SWEEP_STEP_MS + POST_BLACK_MS;

        let mut leds = lit_leds();
        let status = sequence.update(ramp_start + 500);
        sequence.apply_physical(status, &mut leds);
        assert_eq!(leds[0].to_srgb8(), [0, 0, 0]);
        assert_eq!(leds[1].to_srgb8(), [102, 127, 153]);
        assert_eq!(leds[2].to_srgb8(), [0, 0, 0]);

        let mut leds = lit_leds();
        let status = sequence.update(ramp_start + POST_NEGOTIATION_MAX_MS);
        sequence.apply_physical(status, &mut leds);
        assert!(leds.iter().all(|pixel| pixel.to_srgb8() == [102, 127, 153]));
    }
}
