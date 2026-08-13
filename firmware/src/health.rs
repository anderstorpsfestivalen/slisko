//! Shared runtime health state and the allocation-only JSON representation used
//! by the HTTP status endpoint.

use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceState {
    Starting,
    Running,
    Retrying,
    Stopped,
}

impl ServiceState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Retrying => "retrying",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HealthSnapshot {
    pub boot_ms: u64,
    pub frames: u32,
    pub consecutive_output_errors: u32,
    pub total_output_errors: u32,
    pub ethernet_driver: &'static str,
    pub ethernet_link: &'static str,
    pub dhcp: &'static str,
    pub ip: Option<String>,
    pub dhcp_repair_attempts: u32,
    pub ntp_state: &'static str,
    pub ntp_ever_synced: bool,
    pub ntp_last_sync_ms: Option<u64>,
    pub ntp_restart_attempts: u32,
    pub http: ServiceState,
    pub ddp: ServiceState,
    pub mdns: ServiceState,
    pub buttons: ServiceState,
    pub last_error: Option<String>,
}

impl HealthSnapshot {
    fn new(boot_ms: u64) -> Self {
        Self {
            boot_ms,
            frames: 0,
            consecutive_output_errors: 0,
            total_output_errors: 0,
            ethernet_driver: "starting",
            ethernet_link: "unknown",
            dhcp: "waiting",
            ip: None,
            dhcp_repair_attempts: 0,
            ntp_state: "waiting_for_network",
            ntp_ever_synced: false,
            ntp_last_sync_ms: None,
            ntp_restart_attempts: 0,
            http: ServiceState::Starting,
            ddp: ServiceState::Starting,
            mdns: ServiceState::Starting,
            buttons: ServiceState::Starting,
            last_error: None,
        }
    }

    pub fn to_json(
        &self,
        now_ms: u64,
        unix_time_s: Option<u64>,
        ddp_enabled: bool,
        ddp_active: bool,
    ) -> String {
        let uptime_s = now_ms.saturating_sub(self.boot_ms) / 1_000;
        let last_sync_age = self
            .ntp_last_sync_ms
            .map(|sync| now_ms.saturating_sub(sync) / 1_000);
        let unix_time_s = self.ntp_ever_synced.then_some(unix_time_s).flatten();
        let healthy = self.consecutive_output_errors == 0
            && self.ethernet_driver == "started"
            && self.ethernet_link == "up"
            && self.dhcp == "leased"
            && self.ntp_state == "synced"
            && self.http == ServiceState::Running
            && self.ddp == ServiceState::Running
            && self.mdns == ServiceState::Running
            && self.buttons == ServiceState::Running;

        format!(
            concat!(
                "{{\"healthy\":{},\"uptime_s\":{},",
                "\"source\":{{\"mode\":\"{}\",\"rendering\":\"{}\"}},",
                "\"render\":{{\"frames\":{},\"consecutive_output_errors\":{},\"total_output_errors\":{}}},",
                "\"ethernet\":{{\"driver\":\"{}\",\"link\":\"{}\",\"dhcp\":\"{}\",\"ip\":{},\"repair_attempts\":{}}},",
                "\"ntp\":{{\"state\":\"{}\",\"ever_synced\":{},\"time_unix_s\":{},\"last_sync_age_s\":{},\"restart_attempts\":{}}},",
                "\"services\":{{\"http\":\"{}\",\"ddp\":\"{}\",\"mdns\":\"{}\",\"buttons\":\"{}\"}},",
                "\"last_error\":{}}}"
            ),
            healthy,
            uptime_s,
            if ddp_enabled { "ddp" } else { "internal" },
            if ddp_active { "ddp" } else { "internal" },
            self.frames,
            self.consecutive_output_errors,
            self.total_output_errors,
            self.ethernet_driver,
            self.ethernet_link,
            self.dhcp,
            json_option(self.ip.as_deref()),
            self.dhcp_repair_attempts,
            self.ntp_state,
            self.ntp_ever_synced,
            unix_time_s.map_or_else(|| "null".into(), |time| time.to_string()),
            last_sync_age.map_or_else(|| "null".into(), |age| age.to_string()),
            self.ntp_restart_attempts,
            self.http.as_str(),
            self.ddp.as_str(),
            self.mdns.as_str(),
            self.buttons.as_str(),
            json_option(self.last_error.as_deref()),
        )
    }
}

#[derive(Clone)]
pub struct Health {
    inner: Arc<Mutex<HealthSnapshot>>,
}

impl Health {
    pub fn new(boot_ms: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HealthSnapshot::new(boot_ms))),
        }
    }

    pub fn update(&self, update: impl FnOnce(&mut HealthSnapshot)) {
        update(&mut lock_recover(&self.inner));
    }

    pub fn snapshot(&self) -> HealthSnapshot {
        lock_recover(&self.inner).clone()
    }

    pub fn record_error(&self, message: impl Into<String>) {
        self.update(|health| health.last_error = Some(message.into()));
    }
}

pub fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            mutex.clear_poison();
            poisoned.into_inner()
        }
    }
}

fn json_option(value: Option<&str>) -> String {
    value.map_or_else(
        || "null".into(),
        |value| format!("\"{}\"", escape_json(value)),
    )
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push('?'),
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_json_reports_degraded_and_escapes_errors() {
        let mut health = HealthSnapshot::new(1_000);
        health.last_error = Some("bad \"link\"\n".into());
        let json = health.to_json(4_000, Some(1_000), true, false);
        assert!(json.contains("\"healthy\":false"));
        assert!(json.contains("\"uptime_s\":3"));
        assert!(json.contains("\"time_unix_s\":null"));
        assert!(json.contains("\"source\":{\"mode\":\"ddp\",\"rendering\":\"internal\"}"));
        assert!(json.contains("\"mdns\":\"starting\""));
        assert!(json.contains("bad \\\"link\\\"\\n"));
    }

    #[test]
    fn health_json_becomes_healthy_when_every_service_is_ready() {
        let mut health = HealthSnapshot::new(0);
        health.ethernet_driver = "started";
        health.ethernet_link = "up";
        health.dhcp = "leased";
        health.ntp_state = "synced";
        health.ntp_ever_synced = true;
        health.http = ServiceState::Running;
        health.ddp = ServiceState::Running;
        health.mdns = ServiceState::Running;
        health.buttons = ServiceState::Running;
        assert!(
            health
                .to_json(0, Some(1_000), false, false)
                .contains("\"healthy\":true")
        );
        assert!(
            health
                .to_json(0, Some(1_000), false, false)
                .contains("\"time_unix_s\":1000")
        );
    }
}
