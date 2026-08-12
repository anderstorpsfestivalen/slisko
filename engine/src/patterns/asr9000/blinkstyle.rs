use crate::patterns::blinkstyle::{BlinkStyle, ColorStyle};

/// Blink style for ASR9000 40GE linecards.
pub(crate) fn asr9000_style() -> BlinkStyle {
    BlinkStyle {
        min_interval: 0.05,
        max_interval: 7.0,
        min_blink: 0.05,
        max_blink: 12.0,
        min_blinks: 15.0,
        max_blinks: 40.0,
        min_cycle: 1.0,
        max_cycle: 10.0,
        slow_color: ColorStyle {
            r: 1.0,
            g: 0.6,
            b: 0.0,
        },
        fast_color: ColorStyle {
            r: 0.3,
            g: 1.0,
            b: 0.0,
        },
        dead_color: ColorStyle {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        dead_port_chance: 0.067,
        slow_speed_chance: 0.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_port_probability_is_unchanged() {
        assert_eq!(asr9000_style().dead_port_chance, 0.067);
    }
}
