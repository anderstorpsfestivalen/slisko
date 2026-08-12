use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

/// ASR9000 debug pattern that lights cards by role to verify the physical map.
#[derive(Default)]
pub struct Mapper;

impl Pattern for Mapper {
    fn render(&mut self, _info: &RenderInfo, c: &mut Chassi) {
        let mut status = Vec::new();
        let mut links = Vec::new();
        let mut rsp_sync = Vec::new();

        for linecard in &c.linecards {
            match linecard.name {
                "A9K-40GE-L" | "A9K-8T-L" => {
                    if let Some(status_index) = linecard.status {
                        status.push(status_index);
                    }
                    links.extend_from_slice(&linecard.link);
                }
                "A9K-RSP440-SE" | "A9K-RSP440-SE-2" if linecard.led_count > 10 => {
                    rsp_sync.push(linecard.led_offset + 10);
                }
                _ => {}
            }
        }

        for index in status {
            c.leds[index].set_clamped(0.0, 1.0, 0.0);
        }
        for index in links {
            c.leds[index].set_clamped(1.0, 0.0, 0.0);
        }
        for index in rsp_sync {
            c.leds[index].set_clamped(0.0, 1.0, 0.0);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "mapper",
            category: "global",
        }
    }

    fn bootstrap(&mut self, _c: &Chassi, _ctx: &mut BootstrapCtx) {}
}
