//! Small, platform-neutral recovery state machines used by the firmware.

use config::ButtonAction;

#[derive(Clone, Copy, Debug)]
pub struct ExponentialBackoff {
    initial_ms: u64,
    maximum_ms: u64,
    current_ms: u64,
    retry_at_ms: u64,
}

impl ExponentialBackoff {
    pub const fn new(initial_ms: u64, maximum_ms: u64) -> Self {
        Self {
            initial_ms,
            maximum_ms,
            current_ms: initial_ms,
            retry_at_ms: 0,
        }
    }

    pub fn ready(&self, now_ms: u64) -> bool {
        now_ms >= self.retry_at_ms
    }

    pub fn fail(&mut self, now_ms: u64) -> u64 {
        let delay = self.current_ms;
        self.retry_at_ms = now_ms.saturating_add(delay);
        self.current_ms = self.current_ms.saturating_mul(2).min(self.maximum_ms);
        delay
    }

    pub fn reset(&mut self) {
        self.current_ms = self.initial_ms;
        self.retry_at_ms = 0;
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FailureWindow {
    first_failure_ms: Option<u64>,
    consecutive: u32,
}

impl FailureWindow {
    pub fn record_failure(&mut self, now_ms: u64) {
        self.first_failure_ms.get_or_insert(now_ms);
        self.consecutive = self.consecutive.saturating_add(1);
    }

    pub fn record_success(&mut self) {
        self.first_failure_ms = None;
        self.consecutive = 0;
    }

    pub fn consecutive(&self) -> u32 {
        self.consecutive
    }

    pub fn expired(&self, now_ms: u64, threshold_ms: u64) -> bool {
        self.first_failure_ms
            .is_some_and(|started| now_ms.saturating_sub(started) >= threshold_ms)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ButtonEdge {
    Pressed,
    Released,
}

/// Time-based active-low button debounce. An edge is emitted only after the
/// raw input remains unchanged for the configured interval.
#[derive(Clone, Copy, Debug)]
pub struct ActiveLowDebouncer {
    debounce_ms: u64,
    sampled_low: bool,
    stable_low: bool,
    sample_changed_ms: u64,
}

impl ActiveLowDebouncer {
    pub const fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            sampled_low: false,
            stable_low: false,
            sample_changed_ms: 0,
        }
    }

    pub fn update(&mut self, low: bool, now_ms: u64) -> Option<ButtonEdge> {
        if low != self.sampled_low {
            self.sampled_low = low;
            self.sample_changed_ms = now_ms;
        }
        if self.sampled_low == self.stable_low
            || now_ms.saturating_sub(self.sample_changed_ms) < self.debounce_ms
        {
            return None;
        }

        self.stable_low = self.sampled_low;
        Some(if self.stable_low {
            ButtonEdge::Pressed
        } else {
            ButtonEdge::Released
        })
    }
}

struct MomentaryScene {
    button_index: usize,
    restore_patterns: Vec<&'static str>,
}

/// Tracks which scene a momentary button must restore. A later non-momentary
/// press cancels that restoration so releasing an older button cannot undo the
/// newly selected scene.
#[derive(Default)]
pub struct ButtonSceneState {
    momentary: Option<MomentaryScene>,
}

impl ButtonSceneState {
    pub fn press(
        &mut self,
        button_index: usize,
        action: ButtonAction,
        current_patterns: &[&'static str],
    ) {
        let previous = self.momentary.take();
        if action == ButtonAction::Momentary {
            self.momentary = Some(MomentaryScene {
                button_index,
                restore_patterns: previous.map_or_else(
                    || current_patterns.to_vec(),
                    |active| active.restore_patterns,
                ),
            });
        }
    }

    pub fn release(&mut self, button_index: usize) -> Option<Vec<&'static str>> {
        self.momentary
            .take_if(|active| active.button_index == button_index)
            .map(|active| active.restore_patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_caps_and_resets() {
        let mut retry = ExponentialBackoff::new(1_000, 8_000);
        assert!(retry.ready(0));
        assert_eq!(retry.fail(0), 1_000);
        assert!(!retry.ready(999));
        assert!(retry.ready(1_000));
        assert_eq!(retry.fail(1_000), 2_000);
        assert_eq!(retry.fail(3_000), 4_000);
        assert_eq!(retry.fail(7_000), 8_000);
        assert_eq!(retry.fail(15_000), 8_000);
        retry.reset();
        assert!(retry.ready(0));
        assert_eq!(retry.fail(0), 1_000);
    }

    #[test]
    fn failure_window_only_expires_while_failures_are_continuous() {
        let mut failures = FailureWindow::default();
        failures.record_failure(100);
        failures.record_failure(200);
        assert_eq!(failures.consecutive(), 2);
        assert!(!failures.expired(1_099, 1_000));
        assert!(failures.expired(1_100, 1_000));
        failures.record_success();
        assert_eq!(failures.consecutive(), 0);
        assert!(!failures.expired(10_000, 1_000));
    }

    #[test]
    fn button_debounce_rejects_bounce_and_emits_one_edge_per_stable_change() {
        let mut button = ActiveLowDebouncer::new(30);
        assert_eq!(button.update(true, 0), None);
        assert_eq!(button.update(false, 10), None);
        assert_eq!(button.update(true, 20), None);
        assert_eq!(button.update(true, 49), None);
        assert_eq!(button.update(true, 50), Some(ButtonEdge::Pressed));
        assert_eq!(button.update(true, 500), None);
        assert_eq!(button.update(false, 510), None);
        assert_eq!(button.update(false, 540), Some(ButtonEdge::Released));
        assert_eq!(button.update(false, 1_000), None);
    }

    #[test]
    fn momentary_scene_restores_only_if_it_is_still_the_active_override() {
        let defaults = ["greenstatus", "a9k-8t-l"];
        let mut scenes = ButtonSceneState::default();

        scenes.press(0, ButtonAction::Momentary, &defaults);
        assert_eq!(scenes.release(0).unwrap(), defaults);

        scenes.press(0, ButtonAction::Momentary, &defaults);
        scenes.press(1, ButtonAction::Hold, &["lamp-test"]);
        assert_eq!(scenes.release(0), None);
    }

    #[test]
    fn nested_momentary_scenes_preserve_the_original_scene() {
        let defaults = ["greenstatus"];
        let mut scenes = ButtonSceneState::default();
        scenes.press(0, ButtonAction::Momentary, &defaults);
        scenes.press(1, ButtonAction::Momentary, &["lamp-test"]);
        assert_eq!(scenes.release(0), None);
        assert_eq!(scenes.release(1).unwrap(), defaults);
    }
}
