use alloc::vec::Vec;

use super::blinkstyle::asr9000_uplink_style;
use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, LinkActivation, Pattern, PatternInfo, RenderInfo};
use crate::patterns::blinkstyle::PortState;

/// ASR9000 A9K-8T-L uplinks. The existing green/amber activity presentation
/// and roughly 1-in-30 solid-red population are preserved.
#[derive(Default)]
pub struct A9K8TL {
    ports: Vec<PortState>,
}

impl Pattern for A9K8TL {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for port in &mut self.ports {
            port.render(&mut c.leds, info);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "a9k-8t-l",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        let style = asr9000_uplink_style();
        for index in c.link_indices_of_type("A9K-8T-L") {
            self.ports.push(style.create_port(index, ctx));
        }
    }

    fn restart_link_traffic(&mut self, now_ms: u32, activations: &[LinkActivation]) {
        for port in &mut self.ports {
            port.restart_link_traffic(now_ms, activations);
        }
    }
}
