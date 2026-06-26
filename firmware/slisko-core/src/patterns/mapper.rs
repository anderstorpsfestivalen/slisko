//! Ported from `patterns/mapper.go` — a debug pattern that lights cards by role
//! (status green, link red, RSP "sync" LED green) to verify the physical map.

use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};

#[derive(Default)]
pub struct Mapper;

impl Pattern for Mapper {
    fn render(&mut self, _info: &RenderInfo, c: &mut Chassi) {
        let mut status = Vec::new();
        let mut links = Vec::new();
        let mut rsp_sync = Vec::new();

        for lc in &c.linecards {
            match lc.name {
                "A9K-40GE-L" | "A9K-8T-L" => {
                    if let Some(s) = lc.status {
                        status.push(s);
                    }
                    links.extend_from_slice(&lc.link);
                }
                // Go lit LEDs[10] (the "sync" LED) green.
                "A9K-RSP440-SE" | "A9K-RSP440-SE-2" if lc.led_count > 10 => {
                    rsp_sync.push(lc.led_offset + 10);
                }
                _ => {}
            }
        }

        for i in status {
            c.leds[i].set_clamped(0.0, 1.0, 0.0);
        }
        for i in links {
            c.leds[i].set_clamped(1.0, 0.0, 0.0);
        }
        for i in rsp_sync {
            c.leds[i].set_clamped(0.0, 1.0, 0.0);
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
