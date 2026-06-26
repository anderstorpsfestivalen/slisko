//! Color helpers. `hsv` mirrors `go-colorful`'s `Hsv(h, s, v)` (standard
//! HSV→sRGB), which the Go patterns use via `colorful.Hsv`.

use libm::{fabsf, fmodf};

/// HSV to RGB. `h` in degrees `[0, 360)`, `s`/`v` in `[0, 1]`. Returns sRGB
/// channels in `[0, 1]`.
pub fn hsv(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let hp = fmodf(h, 360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - fabsf(fmodf(hp, 2.0) - 1.0));
    let m = v - c;

    let (r1, g1, b1) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r1 + m, g1 + m, b1 + m)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: (f32, f32, f32), b: (f32, f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-5 && (a.1 - b.1).abs() < 1e-5 && (a.2 - b.2).abs() < 1e-5
    }

    #[test]
    fn primaries() {
        assert!(close(hsv(0.0, 1.0, 1.0), (1.0, 0.0, 0.0))); // red
        assert!(close(hsv(120.0, 1.0, 1.0), (0.0, 1.0, 0.0))); // green
        assert!(close(hsv(240.0, 1.0, 1.0), (0.0, 0.0, 1.0))); // blue
        assert!(close(hsv(0.0, 0.0, 1.0), (1.0, 1.0, 1.0))); // white
        assert!(close(hsv(0.0, 0.0, 0.0), (0.0, 0.0, 0.0))); // black
    }
}
