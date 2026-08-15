use alloc::vec::Vec;

use super::blinkstyle::cisco7609_uplink_style;
use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, LinkActivation, Pattern, PatternInfo, RenderInfo};
use crate::patterns::blinkstyle::PortState;

/// Cisco 7609 6704 uplinks: predominantly heavy core traffic with the existing
/// green-link and solid-red dead-port population.
#[derive(Default)]
pub struct X6704 {
    ports: Vec<PortState>,
}

impl Pattern for X6704 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for port in &mut self.ports {
            port.render(&mut c.leds, info);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "x6704",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        let style = cisco7609_uplink_style();
        for index in c.link_indices_of_type("6704") {
            self.ports.push(style.create_port(index, ctx));
        }
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
    use crate::faker::Rng;
    use crate::pixel::Position;

    static POSITION: &[Position] = &[Position {
        x: 0.0,
        y: 0.0,
        size: 1.0,
    }];
    static SPEC: &[LineCardSpec] = &[LineCardSpec {
        name: "6704",
        image: "",
        active: true,
        positions: POSITION,
        link: &[0],
        status: None,
        labeled: &[],
    }];

    #[test]
    fn dead_port_stays_solid_red() {
        let mut chassis = Chassi::from_specs(SPEC);
        let mut style = cisco7609_uplink_style();
        style.dead_port_chance = 1.0;
        let mut rng = Rng::new(7);
        let mut pattern = X6704 {
            ports: vec![style.create_port(0, &mut BootstrapCtx { rng: &mut rng })],
        };

        pattern.render(&RenderInfo::default(), &mut chassis);
        assert_eq!(chassis.leds[0].to_srgb8(), [255, 0, 0]);
        pattern.render(
            &RenderInfo {
                millis: 60_000,
                ..RenderInfo::default()
            },
            &mut chassis,
        );
        assert_eq!(chassis.leds[0].to_srgb8(), [255, 0, 0]);
    }
}
