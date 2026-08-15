use crate::patterns::blinkstyle::{BlinkStyle, ColorStyle};

/// Cisco 7609 link LEDs use solid red for down/dead ports. Healthy ports are
/// green for gigabit or orange for 100 Mbit/s.
pub(crate) const DEAD_PORT_CHANCE: f32 = 0.03;
pub(crate) const HEALTHY_COLOR: ColorStyle = ColorStyle {
    r: 0.0,
    g: 1.0,
    b: 0.0,
};
pub(crate) const COLOR_100MBIT: ColorStyle = ColorStyle {
    r: 1.0,
    g: 0.8,
    b: 0.0,
};
pub(crate) const PORT_100MBIT_CHANCE: f32 = 0.05;
const SURVIVOR_100MBIT_CHANCE: f32 = PORT_100MBIT_CHANCE / (1.0 - DEAD_PORT_CHANCE);

pub(crate) fn cisco7609_style() -> BlinkStyle {
    BlinkStyle {
        min_interval: 0.1,
        max_interval: 7.0,
        min_blink: 0.1,
        max_blink: 12.0,
        min_blinks: 15.0,
        max_blinks: 40.0,
        min_cycle: 1.0,
        max_cycle: 10.0,
        slow_color: COLOR_100MBIT,
        fast_color: HEALTHY_COLOR,
        dead_color: ColorStyle {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        },
        dead_port_chance: DEAD_PORT_CHANCE,
        slow_speed_chance: SURVIVOR_100MBIT_CHANCE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faker::Rng;
    use crate::pattern::BootstrapCtx;
    use crate::pixel::Pixel;

    #[test]
    fn port_populations_and_colors_match_link_semantics() {
        let style = cisco7609_style();
        assert_eq!(style.dead_port_chance, 0.03);
        assert!(((1.0 - style.dead_port_chance) * style.slow_speed_chance - 0.05).abs() < 1e-6);
        assert_eq!(style.slow_color.r, 1.0);
        assert_eq!(style.slow_color.g, 0.8);
        assert_eq!(style.slow_color.b, 0.0);
        assert_eq!(style.fast_color.r, 0.0);
        assert_eq!(style.fast_color.g, 1.0);
        assert_eq!(style.fast_color.b, 0.0);
    }

    #[test]
    fn dead_ports_are_solid_red() {
        let mut style = cisco7609_style();
        style.dead_port_chance = 1.0;
        let mut rng = Rng::new(7);
        let mut ctx = BootstrapCtx { rng: &mut rng };
        let mut port = style.create_port(0, &mut ctx);
        let mut leds = [Pixel::new()];

        port.render(&mut leds, 0);
        assert_eq!(leds[0].to_srgb8(), [255, 0, 0]);
        port.render(&mut leds, 60_000);
        assert_eq!(leds[0].to_srgb8(), [255, 0, 0]);
    }
}
