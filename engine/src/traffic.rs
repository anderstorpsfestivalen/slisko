//! Ported from `pkg/traffic/shaper.go`.
//!
//! Time-of-day intensity curve. The Go version reads `time.Now()`; here
//! [`Shaper::intensity`] takes the fractional hour-of-day (0.0..24.0) so the
//! firmware feeds it from SNTP and the host from its clock.

use crate::math;

/// Config mirrors `configuration.TrafficShaper` (hours are 0..23).
#[derive(Clone, Copy, Debug)]
pub struct ShaperConfig {
    pub enabled: bool,
    /// POSIX timezone used by runtimes when converting SNTP/system time into
    /// the local hour passed to [`Shaper::intensity`].
    pub timezone: &'static str,
    pub peak_start: f32,
    pub peak_end: f32,
    pub low_start: f32,
    pub low_end: f32,
    pub peak_factor: f32,
    pub low_factor: f32,
}

impl Default for ShaperConfig {
    /// Matches `configuration.DefaultTrafficShaper`.
    fn default() -> Self {
        ShaperConfig {
            enabled: true,
            timezone: "UTC0",
            peak_start: 17.0,
            peak_end: 22.0,
            low_start: 2.0,
            low_end: 7.0,
            peak_factor: 1.0,
            low_factor: 0.2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Shaper {
    cfg: ShaperConfig,
}

impl Shaper {
    pub fn new(cfg: ShaperConfig) -> Self {
        Shaper { cfg }
    }

    /// Intensity multiplier for the given local fractional hour-of-day.
    ///
    /// The configured low and peak windows are plateaus. The two gaps between
    /// them use half-cosine ramps, which keeps the curve and its slope
    /// continuous at every boundary, including across midnight.
    pub fn intensity(&self, hour_of_day: f32) -> f32 {
        if !self.cfg.enabled {
            return 1.0;
        }

        let low_len = forward_hours(self.cfg.low_start, self.cfg.low_end);
        let rise_len = forward_hours(self.cfg.low_end, self.cfg.peak_start);
        let peak_len = forward_hours(self.cfg.peak_start, self.cfg.peak_end);
        let fall_len = forward_hours(self.cfg.peak_end, self.cfg.low_start);
        let elapsed = forward_hours(self.cfg.low_start, hour_of_day);

        if elapsed <= low_len {
            return self.cfg.low_factor;
        }
        if elapsed < low_len + rise_len {
            return smooth_between(
                self.cfg.low_factor,
                self.cfg.peak_factor,
                (elapsed - low_len) / rise_len,
            );
        }
        if elapsed <= low_len + rise_len + peak_len {
            return self.cfg.peak_factor;
        }

        smooth_between(
            self.cfg.peak_factor,
            self.cfg.low_factor,
            (elapsed - low_len - rise_len - peak_len) / fall_len,
        )
    }

    /// Scale a base duration (seconds) by intensity: higher intensity → shorter
    /// (mirrors `GetScaledDuration`, with the same 0.1 floor).
    pub fn scaled_secs(&self, base: f32, hour_of_day: f32) -> f32 {
        let i = self.intensity(hour_of_day).max(0.1);
        base / i
    }

    /// Scale a (min, max) interval pair (seconds) by intensity (mirrors
    /// `GetScaledInterval`).
    pub fn scaled_interval(&self, min: f32, max: f32, hour_of_day: f32) -> (f32, f32) {
        let i = self.intensity(hour_of_day).max(0.1);
        (min / i, max / i)
    }
}

fn forward_hours(start: f32, end: f32) -> f32 {
    let wrapped = (end - start) % 24.0;
    if wrapped < 0.0 {
        wrapped + 24.0
    } else {
        wrapped
    }
}

fn smooth_between(start: f32, end: f32, progress: f32) -> f32 {
    let eased = (1.0 - math::cosf(progress.clamp(0.0, 1.0) * core::f32::consts::PI)) * 0.5;
    start + (end - start) * eased
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_is_unity() {
        let s = Shaper::new(ShaperConfig {
            enabled: false,
            ..Default::default()
        });
        assert_eq!(s.intensity(3.0), 1.0);
    }

    #[test]
    fn configured_windows_are_plateaus() {
        let s = Shaper::default();
        for hour in [2.0, 4.5, 7.0] {
            assert!((s.intensity(hour) - 0.2).abs() < 1e-6);
        }
        for hour in [17.0, 19.5, 22.0] {
            assert!((s.intensity(hour) - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn transitions_are_smooth_and_wrap_midnight() {
        let s = Shaper::default();
        let morning = s.intensity(12.0);
        let late_evening = s.intensity(23.0);
        let after_midnight = s.intensity(1.0);
        assert!(morning > 0.2 && morning < 1.0);
        assert!(late_evening > after_midnight);
        assert!(after_midnight > 0.2);
        assert!((s.intensity(7.0) - s.intensity(7.0001)).abs() < 1e-4);
        assert!((s.intensity(22.0) - s.intensity(22.0001)).abs() < 1e-4);
    }

    #[test]
    fn scaling_shortens_with_intensity() {
        let s = Shaper::default();
        // At peak (intensity ~1.0) a 10s base stays ~10s; at trough (~0.2) it grows.
        assert!(s.scaled_secs(10.0, 19.5) < s.scaled_secs(10.0, 7.5));
    }
}
