//! Platform-neutral redundant-power state tracking and final-frame overrides.

use crate::pixel::Pixel;

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
    sampled_low: bool,
    stable_low: bool,
    sample_changed_ms: u64,
}

impl ActiveLowDebouncer {
    const fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            sampled_low: false,
            stable_low: false,
            sample_changed_ms: 0,
        }
    }

    fn update(&mut self, low: bool, now_ms: u64) -> bool {
        if low != self.sampled_low {
            self.sampled_low = low;
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
}

impl RedundantPowerMonitor {
    pub const fn new(debounce_ms: u64) -> Self {
        Self {
            inputs: [ActiveLowDebouncer::new(debounce_ms); 2],
            state: RedundantPowerState::Offline,
        }
    }

    pub const fn state(&self) -> RedundantPowerState {
        self.state
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
        if !changed {
            return None;
        }

        let next = RedundantPowerState::from_online([
            self.inputs[0].stable_low,
            self.inputs[1].stable_low,
        ]);
        if next == self.state {
            return None;
        }
        self.state = next;
        Some(next)
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
}
