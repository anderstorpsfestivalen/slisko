//! Color helpers. `hsv` mirrors `go-colorful`'s `Hsv(h, s, v)` (standard
//! HSV→sRGB), which the Go patterns use via `colorful.Hsv`.

/// HSV to RGB. `h` in degrees `[0, 360)`, `s`/`v` in `[0, 1]`. Returns sRGB
/// channels in `[0, 1]`.
pub fn hsv(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    // Callers already supply one hue turn. Selecting the HSV sector directly
    // avoids two fmodf calls in Colorcycler's per-frame color lookup.
    let hp = h / 60.0;
    let sector = hp as u8;
    let fraction = hp - f32::from(sector);
    let c = v * s;
    let x = if sector & 1 == 0 {
        c * fraction
    } else {
        c * (1.0 - fraction)
    };
    let m = v - c;

    let (r1, g1, b1) = match sector % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
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
        assert!(close(hsv(360.0, 1.0, 1.0), (1.0, 0.0, 0.0))); // wrap
    }

    #[test]
    fn integer_sectors_match_reference_across_the_hue_wheel() {
        fn reference(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
            let hp = libm::fmodf(h, 360.0) / 60.0;
            let c = v * s;
            let x = c * (1.0 - libm::fabsf(libm::fmodf(hp, 2.0) - 1.0));
            let m = v - c;
            let (r, g, b) = if hp < 1.0 {
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
            (r + m, g + m, b + m)
        }

        for tenth_degree in 0..3_600 {
            let h = tenth_degree as f32 * 0.1;
            assert!(close(hsv(h, 0.73, 0.81), reference(h, 0.73, 0.81)));
        }
    }
}
