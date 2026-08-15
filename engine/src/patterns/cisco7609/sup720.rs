use alloc::boxed::Box;
use alloc::vec::Vec;

use super::blinkstyle::cisco7609_sup_style;
use crate::chassi::Chassi;
use crate::faker::Fake;
use crate::pattern::{BootstrapCtx, LinkActivation, Pattern, PatternInfo, RenderInfo};
use crate::patterns::blinkstyle::PortState;
use crate::patterns::panel_helpers::{random_interval, set_all};

/// Cisco 7609 SUP720 control panel.
#[derive(Default)]
pub struct SUP720 {
    disk0: Option<Box<dyn Fake + Send>>,
    disk1: Option<Box<dyn Fake + Send>>,
    ports: Vec<PortState>,
    system: Vec<usize>,
    active: Vec<usize>,
    mgmt: Vec<usize>,
    disk0_led: Vec<usize>,
    disk1_led: Vec<usize>,
}

impl Pattern for SUP720 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let disk0 = self
            .disk0
            .as_mut()
            .map(|faker| faker.trig(info.millis))
            .unwrap_or(0.0);
        let disk1 = self
            .disk1
            .as_mut()
            .map(|faker| faker.trig(info.millis))
            .unwrap_or(0.0);

        set_all(c, &self.system, 0.2, 1.0, 0.0);
        set_all(c, &self.active, 0.2, 1.0, 0.0);
        set_all(c, &self.mgmt, 0.2, 1.0, 0.0);
        set_all(c, &self.disk0_led, 0.0, disk0, 0.0);
        set_all(c, &self.disk1_led, 0.0, disk1, 0.0);
        for port in &mut self.ports {
            port.render(&mut c.leds, info);
        }
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
        let style = cisco7609_sup_style();
        for index in c.link_indices_of_type("sup720") {
            self.ports.push(style.create_port(index, ctx));
        }

        let card_type = "sup720";
        self.system = c.leds_with_label_on_type(card_type, "system");
        self.active = c.leds_with_label_on_type(card_type, "active");
        self.mgmt = c.leds_with_label_on_type(card_type, "mgmt");
        self.disk0_led = c.leds_with_label_on_type(card_type, "disk0");
        self.disk1_led = c.leds_with_label_on_type(card_type, "disk1");
    }

    fn restart_link_traffic(&mut self, now_ms: u32, activations: &[LinkActivation]) {
        for port in &mut self.ports {
            port.restart_link_traffic(now_ms, activations);
        }
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
        Position {
            x: 3.0,
            y: 0.0,
            size: 1.0,
        },
    ];
    static SPEC: &[LineCardSpec] = &[LineCardSpec {
        name: "sup720",
        image: "",
        active: true,
        positions: POSITIONS,
        link: &[1, 2, 3],
        status: None,
        labeled: &[("mgmt", 0)],
    }];

    #[test]
    fn all_three_sup_links_are_rendered() {
        let mut chassis = Chassi::from_specs(SPEC);
        let mut pattern = SUP720::default();
        pattern.mgmt.push(0);
        let mut rng = crate::faker::Rng::new(7);
        pattern.bootstrap(&chassis, &mut BootstrapCtx { rng: &mut rng });

        pattern.render(&RenderInfo::default(), &mut chassis);
        assert_eq!(chassis.leds[0].to_srgb8(), [51, 255, 0]);
        assert!(
            chassis.leds[1..=3]
                .iter()
                .all(|pixel| pixel.to_srgb8() == [0, 255, 0])
        );
    }
}
