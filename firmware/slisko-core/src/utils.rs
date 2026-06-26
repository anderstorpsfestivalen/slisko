//! Ported from `pkg/utils` (`utils.go` + the time-math helpers).
//!
//! The Go time-based helpers take a `time.Time` start and call
//! `time.Since(t).Seconds()` internally. Here they take elapsed seconds
//! directly (`secs: f32`) so the same code drives the firmware (`esp_timer`)
//! and the host simulator without any clock dependency.

use libm::{cosf, fabsf, fmodf, powf, sinf};

/// Branchless `0.0`/`1.0` from a bool, computed in the integer domain and
/// bit-cast to `f32`.
///
/// Background: `if c { 1.0 } else { 0.0 }` (and `(c as u32) as f32`) lower to a
/// `[2 x float]` constant-pool array that the Xtensa backend cannot select
/// (`Cannot select XtensaISD::PCREL_WRAPPER ... constantpool`) at any opt-level.
/// Even computing it as `from_bits(0x3f80_0000 * c as u32)` doesn't help on its
/// own — LLVM recognizes that `0x3f80_0000` is the bit pattern of `1.0_f32` and
/// folds the whole thing back into the float select. So we hide the constant
/// behind a `black_box` optimization barrier, which defeats that fold.
#[inline]
fn bool_to_f32(c: bool) -> f32 {
    f32::from_bits(core::hint::black_box(0x3f80_0000u32) * c as u32)
}

// ---- pure value helpers (utils.go) ----

/// `Crush`: saturate to 1.0 once past the threshold.
pub fn crush(v: f32, threshold: f32) -> f32 {
    if v > threshold { 1.0 } else { v }
}

/// `Threshold`: pass through above the threshold, else 0.
pub fn threshold(v: f32, threshold: f32) -> f32 {
    if v > threshold { v } else { 0.0 }
}

/// `Trigger`: gate a value on a boolean.
pub fn trigger(v: f32, on: bool) -> f32 {
    if on { v } else { 0.0 }
}

/// `Square`: 1.0 for positive input, else 0.0.
pub fn square(v: f32) -> f32 {
    bool_to_f32(v > 0.0)
}

/// `DutyCycle`: 1.0 once `v` passes `-1 + len*2`, else 0.0.
pub fn duty_cycle(v: f32, len: f32) -> f32 {
    bool_to_f32(v > -1.0 + (len * 2.0))
}

/// `Invert`: `1 - v`.
pub fn invert(v: f32) -> f32 {
    1.0 - v
}

// ---- time-based helpers (take elapsed seconds) ----

/// `SinFull`: raw sine in `[-1, 1]`.
pub fn sin_full(secs: f32, speed: f32) -> f32 {
    sinf(speed * secs)
}

/// `CosFull`: raw cosine in `[-1, 1]`.
pub fn cos_full(secs: f32, speed: f32) -> f32 {
    cosf(speed * secs)
}

/// `Sin`: sine normalized to `[0, 1]`.
pub fn sin(secs: f32, speed: f32) -> f32 {
    (sinf(speed * secs) + 1.0) / 2.0
}

/// `Cos`: cosine normalized to `[0, 1]`.
pub fn cos(secs: f32, speed: f32) -> f32 {
    (cosf(speed * secs) + 1.0) / 2.0
}

/// `Triangle`: `|(secs mod period) - amplitude|`.
pub fn triangle(secs: f32, period: f32, amplitude: f32) -> f32 {
    fabsf(fmodf(secs, period) - amplitude)
}

/// `CurlyTriangle`: `triangle` raised to `curl`.
pub fn curly_triangle(secs: f32, period: f32, amplitude: f32, curl: f32) -> f32 {
    powf(fabsf(fmodf(secs, period) - amplitude), curl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_helpers() {
        assert_eq!(crush(0.9, 0.5), 1.0);
        assert_eq!(crush(0.3, 0.5), 0.3);
        assert_eq!(threshold(0.9, 0.5), 0.9);
        assert_eq!(threshold(0.3, 0.5), 0.0);
        assert_eq!(trigger(0.7, true), 0.7);
        assert_eq!(trigger(0.7, false), 0.0);
        assert_eq!(square(0.1), 1.0);
        assert_eq!(square(-0.1), 0.0);
        assert_eq!(invert(0.25), 0.75);
    }

    #[test]
    fn sin_is_normalized() {
        // At secs=0 sine is 0 -> normalized 0.5.
        assert!((sin(0.0, 1.0) - 0.5).abs() < 1e-6);
        // Stays within [0, 1].
        for i in 0..100 {
            let v = sin(i as f32 * 0.1, 2.0);
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn triangle_matches_go_formula() {
        // |(2.5 mod 2.0) - 1.0| = |0.5 - 1.0| = 0.5
        assert!((triangle(2.5, 2.0, 1.0) - 0.5).abs() < 1e-6);
    }
}
