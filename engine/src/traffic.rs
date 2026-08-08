//! Ported from `pkg/traffic/shaper.go`.
//!
//! Time-of-day intensity curve. The Go version reads `time.Now()`; here
//! [`Shaper::intensity`] takes the fractional hour-of-day (0.0..24.0) so the
//! firmware feeds it from SNTP and the host from its clock.

use libm::cosf;

/// Config mirrors `configuration.TrafficShaper` (hours are 0..23).
#[derive(Clone, Copy, Debug)]
pub struct ShaperConfig {
    pub enabled: bool,
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

    /// Intensity multiplier for the given fractional hour-of-day (mirrors
    /// `GetIntensity` + `calculateIntensity`). Cosine peaking at `peak_mid`.
    pub fn intensity(&self, hour_of_day: f32) -> f32 {
        if !self.cfg.enabled {
            return 1.0;
        }

        let peak_mid = (self.cfg.peak_start + self.cfg.peak_end) / 2.0;

        let mut shifted = hour_of_day - peak_mid;
        if shifted < 0.0 {
            shifted += 24.0;
        }
        if shifted >= 24.0 {
            shifted -= 24.0;
        }

        let angle = (shifted / 24.0) * 2.0 * core::f32::consts::PI;
        let sine = cosf(angle);

        ((sine + 1.0) / 2.0) * (self.cfg.peak_factor - self.cfg.low_factor) + self.cfg.low_factor
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
    fn peaks_at_peak_mid_and_dips_opposite() {
        let s = Shaper::default(); // peak 17..22 -> mid 19.5
        let peak = s.intensity(19.5);
        let trough = s.intensity((19.5 + 12.0) % 24.0);
        assert!(peak > trough);
        assert!((peak - 1.0).abs() < 1e-3); // peak_factor = 1.0
        assert!((trough - 0.2).abs() < 1e-3); // low_factor = 0.2
    }

    #[test]
    fn scaling_shortens_with_intensity() {
        let s = Shaper::default();
        // At peak (intensity ~1.0) a 10s base stays ~10s; at trough (~0.2) it grows.
        assert!(s.scaled_secs(10.0, 19.5) < s.scaled_secs(10.0, 7.5));
    }
}
