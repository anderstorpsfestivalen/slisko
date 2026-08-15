use alloc::vec::Vec;

use super::blinkstyle::cisco7609_style;
use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, LinkActivation, Pattern, PatternInfo, RenderInfo};
use crate::patterns::blinkstyle::PortState;

/// Cisco 7609 6478 linecard ports.
#[derive(Default)]
pub struct Blink48Ports {
    ports: Vec<PortState>,
}

impl Pattern for Blink48Ports {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for port in &mut self.ports {
            port.render(&mut c.leds, info);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "blink48ports",
            category: "link",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        let style = cisco7609_style();
        for index in c.link_indices_of_type("6478") {
            self.ports.push(style.create_port(index, ctx));
        }
    }

    fn restart_link_traffic(&mut self, now_ms: u32, activations: &[LinkActivation]) {
        for port in &mut self.ports {
            port.restart_link_traffic(now_ms, activations);
        }
    }
}
