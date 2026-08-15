//! Supervised SNTP time sync feeding the traffic shaper's configured local
//! hour-of-day.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use esp_idf_svc::sntp::{EspSntp, SntpConf, SyncStatus};
use esp_idf_svc::sys::{EspError, esp_timer_get_time};
use log::{info, warn};

use crate::health::Health;
use crate::net::NetworkStatus;
use crate::recovery::ExponentialBackoff;

const RETRY_INITIAL_MS: u64 = 1_000;
const RETRY_MAX_MS: u64 = 60_000;
const SESSION_SYNC_TIMEOUT_MS: u64 = 10 * 60 * 1_000;

pub struct TimeSync {
    sntp: Option<EspSntp<'static>>,
    health: Health,
    retry: ExponentialBackoff,
    session_started_ms: Option<u64>,
    synced_this_session: Arc<AtomicBool>,
}

impl TimeSync {
    pub fn new(health: Health, timezone: &'static str) -> Self {
        // SAFETY: configuration is baked before firmware startup and rejects
        // interior NULs. This runs before SNTP or render worker threads read
        // the process environment.
        unsafe {
            std::env::set_var("TZ", timezone);
            esp_idf_svc::sys::tzset();
        }
        info!("time: local timezone configured as {timezone}");
        Self {
            sntp: None,
            health,
            retry: ExponentialBackoff::new(RETRY_INITIAL_MS, RETRY_MAX_MS),
            session_started_ms: None,
            synced_this_session: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Advance the SNTP lifecycle without blocking rendering. Returns a valid
    /// configured-local hour once the clock has synchronized at least once.
    pub fn poll(&mut self, now_ms: u64, network: NetworkStatus) -> Option<f32> {
        if !network.ip_up {
            if network.lost_ip {
                self.stop("waiting_for_network");
            } else {
                self.health
                    .update(|health| health.ntp_state = "waiting_for_network");
            }
            return self.current_hour();
        }

        if network.became_up {
            self.stop("starting");
            self.retry.reset();
        }

        if self.sntp.is_none() && self.retry.ready(now_ms) {
            self.try_start(now_ms);
        }

        if let Some(sntp) = &self.sntp {
            let completed = sntp.get_sync_status() == SyncStatus::Completed;
            if completed && !self.synced_this_session.swap(true, Ordering::Relaxed) {
                self.mark_synced(now_ms);
            }

            if !self.synced_this_session.load(Ordering::Relaxed)
                && self.session_started_ms.is_some_and(|started| {
                    now_ms.saturating_sub(started) >= SESSION_SYNC_TIMEOUT_MS
                })
            {
                warn!("sntp: no sync after 10 minutes online; recreating client");
                self.health.update(|health| {
                    health.ntp_restart_attempts = health.ntp_restart_attempts.wrapping_add(1);
                    health.ntp_state = "retrying";
                    health.last_error = Some("SNTP session timed out".into());
                });
                self.sntp.take();
                self.session_started_ms = None;
                self.synced_this_session.store(false, Ordering::Relaxed);
                self.retry.fail(now_ms);
            }
        }

        self.current_hour()
    }

    fn try_start(&mut self, now_ms: u64) {
        self.health.update(|health| health.ntp_state = "starting");
        let health = self.health.clone();
        let synced = self.synced_this_session.clone();
        synced.store(false, Ordering::Relaxed);
        let conf = SntpConf {
            servers: ["0.pool.ntp.org", "1.pool.ntp.org", "2.pool.ntp.org"],
            ..Default::default()
        };

        match EspSntp::new_with_callback(&conf, move |_| {
            synced.store(true, Ordering::Relaxed);
            let now_ms = monotonic_ms();
            health.update(|state| {
                state.ntp_state = "synced";
                state.ntp_ever_synced = true;
                state.ntp_last_sync_ms = Some(now_ms);
            });
            info!("sntp: synchronized; local hour = {:.2}", hour_of_day());
        }) {
            Ok(sntp) => {
                info!("sntp: client started with three pool servers");
                self.sntp = Some(sntp);
                self.session_started_ms = Some(now_ms);
                self.retry.reset();
                self.health.update(|health| health.ntp_state = "syncing");
            }
            Err(error) => self.start_failed(now_ms, error),
        }
    }

    fn start_failed(&mut self, now_ms: u64, error: EspError) {
        let delay = self.retry.fail(now_ms);
        warn!("sntp: start failed ({error:?}); retrying in {delay} ms");
        self.health.update(|health| {
            health.ntp_state = "retrying";
            health.ntp_restart_attempts = health.ntp_restart_attempts.wrapping_add(1);
            health.last_error = Some(format!("SNTP start failed: {error:?}"));
        });
    }

    fn mark_synced(&self, now_ms: u64) {
        self.health.update(|health| {
            health.ntp_state = "synced";
            health.ntp_ever_synced = true;
            health.ntp_last_sync_ms = Some(now_ms);
        });
    }

    fn stop(&mut self, state: &'static str) {
        self.sntp.take();
        self.session_started_ms = None;
        self.synced_this_session.store(false, Ordering::Relaxed);
        self.health.update(|health| health.ntp_state = state);
    }

    fn current_hour(&self) -> Option<f32> {
        self.health.snapshot().ntp_ever_synced.then(hour_of_day)
    }
}

/// Current fractional hour-of-day (0.0..24.0) in the configured POSIX
/// timezone.
pub fn hour_of_day() -> f32 {
    unsafe {
        let t: esp_idf_svc::sys::time_t = esp_idf_svc::sys::time(core::ptr::null_mut());
        let mut tm: esp_idf_svc::sys::tm = core::mem::zeroed();
        esp_idf_svc::sys::localtime_r(&t, &mut tm);
        tm.tm_hour as f32 + tm.tm_min as f32 / 60.0 + tm.tm_sec as f32 / 3600.0
    }
}

fn monotonic_ms() -> u64 {
    unsafe { esp_timer_get_time().max(0) as u64 / 1_000 }
}
