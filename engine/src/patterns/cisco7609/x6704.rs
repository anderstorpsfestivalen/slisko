use alloc::vec::Vec;

use super::blinkstyle::{DEAD_PORT_CHANCE, HEALTHY_COLOR};
use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::patterns::port_faker::{PortFaker, standard_faker};
use crate::utils;

/// Cisco 7609 6704 ports: green flicker with solid-red dead ports.
#[derive(Default)]
pub struct X6704 {
    ports: Vec<PortFaker>,
    dead: Vec<usize>,
}

impl Pattern for X6704 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for port in &mut self.ports {
            let value = utils::invert(port.faker.trig(info.millis));
            c.leds[port.port].set_clamped(
                value * HEALTHY_COLOR.r,
                value * HEALTHY_COLOR.g,
                value * HEALTHY_COLOR.b,
            );
        }
        for &index in &self.dead {
            c.leds[index].set_clamped(1.0, 0.0, 0.0);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "x6704",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        for index in c.link_indices_of_type("6704") {
            if ctx.rng.range_f32(0.0, 1.0) < DEAD_PORT_CHANCE {
                self.dead.push(index);
            } else {
                self.ports.push(PortFaker {
                    faker: standard_faker(0.1, 7.0, 0.1, 12.0, ctx),
                    port: index,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::LineCardSpec;
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
        let mut pattern = X6704::default();
        pattern.dead.push(0);

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
