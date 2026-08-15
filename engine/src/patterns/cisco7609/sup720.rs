use alloc::boxed::Box;
use alloc::vec::Vec;

use super::blinkstyle::HEALTHY_COLOR;
use crate::chassi::Chassi;
use crate::faker::Fake;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::patterns::panel_helpers::{random_interval, set_all};
use crate::utils;

/// Cisco 7609 SUP720 control panel.
#[derive(Default)]
pub struct SUP720 {
    disk0: Option<Box<dyn Fake + Send>>,
    disk1: Option<Box<dyn Fake + Send>>,
    port0: Option<Box<dyn Fake + Send>>,
    port1: Option<Box<dyn Fake + Send>>,
    system: Vec<usize>,
    active: Vec<usize>,
    mgmt: Vec<usize>,
    disk0_led: Vec<usize>,
    disk1_led: Vec<usize>,
    p1: Vec<usize>,
    p2: Vec<usize>,
}

impl Pattern for SUP720 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let disk0 = self
            .disk0
            .as_mut()
            .map(|faker| faker.trig(info.traffic_millis))
            .unwrap_or(0.0);
        let disk1 = self
            .disk1
            .as_mut()
            .map(|faker| faker.trig(info.traffic_millis))
            .unwrap_or(0.0);
        let port0 = utils::invert(
            self.port0
                .as_mut()
                .map(|faker| faker.trig(info.traffic_millis))
                .unwrap_or(0.0),
        );
        let port1 = utils::invert(
            self.port1
                .as_mut()
                .map(|faker| faker.trig(info.traffic_millis))
                .unwrap_or(0.0),
        );

        set_all(c, &self.system, 0.2, 1.0, 0.0);
        set_all(c, &self.active, 0.2, 1.0, 0.0);
        set_all(c, &self.mgmt, 0.2, 1.0, 0.0);
        set_all(c, &self.disk0_led, 0.0, disk0, 0.0);
        set_all(c, &self.disk1_led, 0.0, disk1, 0.0);
        set_all(
            c,
            &self.p1,
            HEALTHY_COLOR.r * port0,
            HEALTHY_COLOR.g * port0,
            HEALTHY_COLOR.b * port0,
        );
        set_all(
            c,
            &self.p2,
            HEALTHY_COLOR.r * port1,
            HEALTHY_COLOR.g * port1,
            HEALTHY_COLOR.b * port1,
        );
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "sup720",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        self.disk0 = Some(random_interval(40.0, 12000.0, 0.1, 6.5, ctx));
        self.disk1 = Some(random_interval(40.0, 12000.0, 0.1, 6.5, ctx));
        self.port0 = Some(random_interval(0.3, 12.0, 0.07, 6.5, ctx));
        self.port1 = Some(random_interval(0.2, 7.0, 0.07, 12.0, ctx));

        let card_type = "sup720";
        self.system = c.leds_with_label_on_type(card_type, "system");
        self.active = c.leds_with_label_on_type(card_type, "active");
        self.mgmt = c.leds_with_label_on_type(card_type, "mgmt");
        self.disk0_led = c.leds_with_label_on_type(card_type, "disk0");
        self.disk1_led = c.leds_with_label_on_type(card_type, "disk1");
        self.p1 = c.leds_with_label_on_type(card_type, "p1");
        self.p2 = c.leds_with_label_on_type(card_type, "p2");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::LineCardSpec;
    use crate::pixel::Position;

    static POSITIONS: &[Position] = &[
        Position {
            x: 0.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 1.0,
            y: 0.0,
            size: 1.0,
        },
        Position {
            x: 2.0,
            y: 0.0,
            size: 1.0,
        },
    ];
    static SPEC: &[LineCardSpec] = &[LineCardSpec {
        name: "sup720",
        image: "",
        active: true,
        positions: POSITIONS,
        link: &[1, 2],
        status: None,
        labeled: &[],
    }];

    #[test]
    fn blinking_ports_have_no_red_channel() {
        let mut chassis = Chassi::from_specs(SPEC);
        let mut pattern = SUP720::default();
        pattern.mgmt.push(0);
        pattern.p1.push(1);
        pattern.p2.push(2);

        pattern.render(&RenderInfo::default(), &mut chassis);
        assert_eq!(chassis.leds[0].to_srgb8(), [51, 255, 0]);
        assert_eq!(chassis.leds[1].to_srgb8(), [0, 255, 0]);
        assert_eq!(chassis.leds[2].to_srgb8(), [0, 255, 0]);
    }
}
