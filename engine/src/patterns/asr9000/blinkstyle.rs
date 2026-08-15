use crate::patterns::blinkstyle::{
    ActivityEffect, ActivityProfile, BlinkStyle, ColorStyle, MillisRange,
};

const HEALTHY: ColorStyle = ColorStyle {
    r: 0.3,
    g: 1.0,
    b: 0.0,
};
const AMBER: ColorStyle = ColorStyle {
    r: 1.0,
    g: 0.8,
    b: 0.0,
};
const RED: ColorStyle = ColorStyle {
    r: 1.0,
    g: 0.0,
    b: 0.0,
};

pub(crate) const DENSE_ACTIVITY: ActivityProfile = ActivityProfile::new(
    [0.50, 0.35, 0.15],
    MillisRange::new(40, 101),
    MillisRange::new(20, 56),
    [0.25, 0.50],
);
pub(crate) const UPLINK_ACTIVITY: ActivityProfile = ActivityProfile::new(
    [0.05, 0.25, 0.70],
    MillisRange::new(40, 101),
    MillisRange::new(20, 56),
    [0.25, 0.50],
);
pub(crate) const MGMT_ACTIVITY: ActivityProfile = ActivityProfile::new(
    [0.80, 0.20, 0.0],
    MillisRange::new(40, 101),
    MillisRange::new(20, 56),
    [0.25, 0.50],
);

/// Blink style for ASR9000 40GE linecards.
pub(crate) fn asr9000_style() -> BlinkStyle {
    BlinkStyle {
        slow_color: ColorStyle {
            r: 1.0,
            g: 0.6,
            b: 0.0,
        },
        fast_color: HEALTHY,
        dead_color: RED,
        dead_port_chance: 0.067,
        slow_speed_chance: 0.2,
        activity: DENSE_ACTIVITY,
        effect: ActivityEffect::Dim,
    }
}

pub(crate) fn asr9000_uplink_style() -> BlinkStyle {
    BlinkStyle {
        slow_color: HEALTHY,
        fast_color: HEALTHY,
        dead_color: RED,
        dead_port_chance: 1.0 / 30.0,
        slow_speed_chance: 0.0,
        activity: UPLINK_ACTIVITY,
        effect: ActivityEffect::Alternate(AMBER),
    }
}

pub(crate) fn asr9000_mgmt_style() -> BlinkStyle {
    BlinkStyle {
        slow_color: HEALTHY,
        fast_color: HEALTHY,
        dead_color: RED,
        dead_port_chance: 0.0,
        slow_speed_chance: 0.0,
        activity: MGMT_ACTIVITY,
        effect: ActivityEffect::Dim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_port_probability_is_unchanged() {
        assert_eq!(asr9000_style().dead_port_chance, 0.067);
    }

    #[test]
    fn dense_uplink_and_management_roles_are_tuned_separately() {
        assert_eq!(asr9000_style().activity.role_weights, [0.50, 0.35, 0.15]);
        assert_eq!(
            asr9000_uplink_style().activity.role_weights,
            [0.05, 0.25, 0.70]
        );
        assert_eq!(
            asr9000_mgmt_style().activity.role_weights,
            [0.80, 0.20, 0.0]
        );
    }
}
