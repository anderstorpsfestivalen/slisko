use alloc::vec::Vec;

use super::blinkstyle::asr9000_style;
use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::patterns::blinkstyle::PortState;

/// ASR9000 A9K-40GE-L linecard ports.
#[derive(Default)]
pub struct A9K40GE {
    ports: Vec<PortState>,
}

impl Pattern for A9K40GE {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for port in &mut self.ports {
            port.render(&mut c.leds, info.traffic_millis);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "a9k-40ge-l",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        let style = asr9000_style();
        for index in c.link_indices_of_type("A9K-40GE-L") {
            self.ports.push(style.create_port(index, ctx));
        }
    }
}
