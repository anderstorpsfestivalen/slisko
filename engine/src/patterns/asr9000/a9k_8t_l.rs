use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::patterns::port_faker::{PortFaker, standard_faker};
use crate::utils;

/// ASR9000 A9K-8T-L ports: binary green/amber, with roughly 1 in 30 dead red.
#[derive(Default)]
pub struct A9K8TL {
    ports: Vec<PortFaker>,
    dead: Vec<usize>,
}

impl Pattern for A9K8TL {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for port in &mut self.ports {
            if utils::invert(port.faker.trig(info.millis)) == 1.0 {
                c.leds[port.port].set_clamped(0.3, 1.0, 0.0);
            } else {
                c.leds[port.port].set_clamped(1.0, 0.8, 0.0);
            }
        }
        for &index in &self.dead {
            c.leds[index].set_clamped(1.0, 0.0, 0.0);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "a9k-8t-l",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        for index in c.link_indices_of_type("A9K-8T-L") {
            if ctx.rng.range_f32(0.0, 30.0) < 1.0 {
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
