//! Linecard port patterns (ported from `patterns/{blink48ports,a9k40gel,x6704,
//! a9k8tl}.go`). Two flavors: full [`BlinkStyle`] ports (dead/slow/fast) and
//! plain per-port fakers.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::faker::{Fake, RandomBlinker, RandomInterval, Rng};
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::patterns::blinkstyle::{BlinkStyle, PortState, asr9000_style, cisco7609_style};
use crate::utils;

/// One port driven by a plain `RandomInterval(RandomBlinker)` faker.
struct PortFaker {
    faker: Box<dyn Fake + Send>,
    port: usize,
}

/// The standard `RandomInterval(RandomBlinker(15,40,1,10))` faker the Cisco/ASR
/// port patterns use, with the given (seconds) interval/blink ranges.
fn standard_faker(
    min_i: f32,
    max_i: f32,
    min_b: f32,
    max_b: f32,
    ctx: &mut BootstrapCtx,
) -> Box<dyn Fake + Send> {
    let blinker = RandomBlinker::new(15.0, 40.0, 1.0, 10.0, 0.0, Rng::new(ctx.rng.next_seed()));
    Box::new(RandomInterval::new(
        min_i,
        max_i,
        min_b,
        max_b,
        Box::new(blinker),
        0.0,
        Rng::new(ctx.rng.next_seed()),
    ))
}

// ---- BlinkStyle-based: Blink48Ports, A9K40GE ----

/// `blink48ports`: Cisco 7609 "6478" linecard ports.
#[derive(Default)]
pub struct Blink48Ports {
    ports: Vec<PortState>,
}

impl Pattern for Blink48Ports {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for p in &mut self.ports {
            p.render(&mut c.leds, info.secs);
        }
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "blink48ports",
            category: "link",
        }
    }
    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        let style: BlinkStyle = cisco7609_style();
        for idx in c.link_indices_of_type("6478") {
            self.ports.push(style.create_port(idx, ctx));
        }
    }
}

/// `a9k-40ge-l`: ASR9000 40GE linecard ports.
#[derive(Default)]
pub struct A9K40GE {
    ports: Vec<PortState>,
}

impl Pattern for A9K40GE {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for p in &mut self.ports {
            p.render(&mut c.leds, info.secs);
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
        for idx in c.link_indices_of_type("A9K-40GE-L") {
            self.ports.push(style.create_port(idx, ctx));
        }
    }
}

// ---- plain per-port fakers: X6704, A9K8TL ----

/// `x6704`: Cisco "6704" ports, simple green flicker.
#[derive(Default)]
pub struct X6704 {
    ports: Vec<PortFaker>,
}

impl Pattern for X6704 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for p in &mut self.ports {
            let v = utils::invert(p.faker.trig(info.secs));
            c.leds[p.port].set_clamped(v * 0.3, v * 1.0, 0.0);
        }
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "x6704",
            category: "misc",
        }
    }
    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        for idx in c.link_indices_of_type("6704") {
            self.ports.push(PortFaker {
                faker: standard_faker(0.1, 7.0, 0.1, 12.0, ctx),
                port: idx,
            });
        }
    }
}

/// `a9k-8t-l`: ASR9000 8x10GE ports — binary green/amber, ~1/30 dead red.
#[derive(Default)]
pub struct A9K8TL {
    ports: Vec<PortFaker>,
    dead: Vec<usize>,
}

impl Pattern for A9K8TL {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        for p in &mut self.ports {
            if utils::invert(p.faker.trig(info.secs)) == 1.0 {
                c.leds[p.port].set_clamped(0.3, 1.0, 0.0);
            } else {
                c.leds[p.port].set_clamped(1.0, 0.8, 0.0);
            }
        }
        for &idx in &self.dead {
            c.leds[idx].set_clamped(1.0, 0.0, 0.0);
        }
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "a9k-8t-l",
            category: "misc",
        }
    }
    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        for idx in c.link_indices_of_type("A9K-8T-L") {
            // ~1 in 30 dead (mirrors rand.Intn(30) == 0).
            if ctx.rng.range_f32(0.0, 30.0) < 1.0 {
                self.dead.push(idx);
            } else {
                self.ports.push(PortFaker {
                    faker: standard_faker(0.1, 7.0, 0.1, 12.0, ctx),
                    port: idx,
                });
            }
        }
    }
}
