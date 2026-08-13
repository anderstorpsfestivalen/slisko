//! Platform-neutral timing policy for Ethernet preference and WiFi attempts.

const WIFI_FALLBACK_DELAY_MS: u64 = 30_000;
const WIFI_ATTEMPT_TIMEOUT_MS: u64 = 15_000;
const WIFI_BETWEEN_ATTEMPTS_MS: u64 = 1_000;
const WIFI_LIST_RETRY_MS: u64 = 5_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackCommand {
    None,
    StartWifi,
    StopWifi,
}

#[derive(Clone, Copy, Debug)]
pub struct FallbackPolicy {
    fallback_at_ms: Option<u64>,
    wifi_requested: bool,
}

impl FallbackPolicy {
    pub fn new(now_ms: u64) -> Self {
        Self {
            fallback_at_ms: Some(now_ms.saturating_add(WIFI_FALLBACK_DELAY_MS)),
            wifi_requested: false,
        }
    }

    pub fn poll(&mut self, now_ms: u64, ethernet_ip_up: bool) -> FallbackCommand {
        if ethernet_ip_up {
            self.fallback_at_ms = None;
            return if core::mem::take(&mut self.wifi_requested) {
                FallbackCommand::StopWifi
            } else {
                FallbackCommand::None
            };
        }

        let fallback_at = *self
            .fallback_at_ms
            .get_or_insert_with(|| now_ms.saturating_add(WIFI_FALLBACK_DELAY_MS));
        if !self.wifi_requested && now_ms >= fallback_at {
            self.wifi_requested = true;
            FallbackCommand::StartWifi
        } else {
            FallbackCommand::None
        }
    }

    pub fn wifi_requested(&self) -> bool {
        self.wifi_requested
    }
}

pub fn wifi_attempt_deadline(now_ms: u64) -> u64 {
    now_ms.saturating_add(WIFI_ATTEMPT_TIMEOUT_MS)
}

pub fn next_credential(current: usize, count: usize, now_ms: u64) -> (usize, u64) {
    if current.saturating_add(1) < count {
        (current + 1, now_ms.saturating_add(WIFI_BETWEEN_ATTEMPTS_MS))
    } else {
        (0, now_ms.saturating_add(WIFI_LIST_RETRY_MS))
    }
}

pub fn credentials_valid(ssid: &str, password: &str) -> bool {
    ssid.len() <= 32 && password.len() <= 64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_waits_thirty_seconds_when_ethernet_is_absent_at_boot() {
        let mut policy = FallbackPolicy::new(1_000);
        assert_eq!(policy.poll(30_999, false), FallbackCommand::None);
        assert_eq!(policy.poll(31_000, false), FallbackCommand::StartWifi);
        assert_eq!(policy.poll(60_000, false), FallbackCommand::None);
    }

    #[test]
    fn ethernet_recovery_cancels_or_stops_fallback() {
        let mut policy = FallbackPolicy::new(0);
        assert_eq!(policy.poll(10_000, true), FallbackCommand::None);
        assert_eq!(policy.poll(40_000, false), FallbackCommand::None);
        assert_eq!(policy.poll(69_999, false), FallbackCommand::None);
        assert_eq!(policy.poll(70_000, false), FallbackCommand::StartWifi);
        assert!(policy.wifi_requested());
        assert_eq!(policy.poll(70_001, true), FallbackCommand::StopWifi);
        assert!(!policy.wifi_requested());
        assert_eq!(policy.poll(80_000, true), FallbackCommand::None);
    }

    #[test]
    fn every_later_ethernet_loss_gets_a_fresh_delay() {
        let mut policy = FallbackPolicy::new(0);
        assert_eq!(policy.poll(0, true), FallbackCommand::None);
        assert_eq!(policy.poll(1_000, false), FallbackCommand::None);
        assert_eq!(policy.poll(30_999, false), FallbackCommand::None);
        assert_eq!(policy.poll(31_000, false), FallbackCommand::StartWifi);
        assert_eq!(policy.poll(32_000, true), FallbackCommand::StopWifi);
        assert_eq!(policy.poll(33_000, false), FallbackCommand::None);
        assert_eq!(policy.poll(63_000, false), FallbackCommand::StartWifi);
    }

    #[test]
    fn credentials_time_out_and_advance_in_order_before_restarting() {
        assert_eq!(wifi_attempt_deadline(10_000), 25_000);
        assert_eq!(next_credential(0, 3, 25_000), (1, 26_000));
        assert_eq!(next_credential(1, 3, 41_000), (2, 42_000));
        assert_eq!(next_credential(2, 3, 57_000), (0, 62_000));
    }

    #[test]
    fn validates_wifi_field_limits_and_open_networks() {
        assert!(credentials_valid("festival", "secret"));
        assert!(credentials_valid("open", ""));
        assert!(!credentials_valid(&"s".repeat(33), "secret"));
        assert!(!credentials_valid("festival", &"p".repeat(65)));
    }
}
