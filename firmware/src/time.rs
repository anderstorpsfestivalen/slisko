//! SNTP time sync, feeding the traffic shaper's hour-of-day.
//!
//! Starts background SNTP (default pool servers) over Ethernet. `hour_of_day`
//! reads the system clock once synced. Timezone is UTC for now (the shaper only
//! needs a consistent day phase); setting TZ is a future refinement.

use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::sys::EspError;

/// Keeps the SNTP client alive.
pub struct TimeSync {
    sntp: EspSntp<'static>,
}

impl TimeSync {
    pub fn start() -> Result<Self, EspError> {
        Ok(TimeSync {
            sntp: EspSntp::new(&Default::default())?,
        })
    }

    pub fn synced(&self) -> bool {
        self.sntp.get_sync_status() == SyncStatus::Completed
    }
}

/// Current fractional hour-of-day (0.0..24.0) from the system clock (UTC).
pub fn hour_of_day() -> f32 {
    unsafe {
        let t: esp_idf_svc::sys::time_t = esp_idf_svc::sys::time(core::ptr::null_mut());
        let mut tm: esp_idf_svc::sys::tm = core::mem::zeroed();
        esp_idf_svc::sys::localtime_r(&t, &mut tm);
        tm.tm_hour as f32 + tm.tm_min as f32 / 60.0
    }
}
