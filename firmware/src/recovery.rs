//! Small, platform-neutral recovery state machines used by the firmware.

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
}
