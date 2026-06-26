//! Labeled control-panel patterns (ported from `patterns/{rsp440,sup720}.go`).
//! These drive named LEDs on a supervisor/RSP card rather than link ports.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::faker::{Fake, RandomBlinker, RandomInterval, Rng};
use crate::pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
use crate::utils;

fn ri(
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

fn set_all(c: &mut Chassi, idxs: &[usize], r: f32, g: f32, b: f32) {
    for &i in idxs {
        c.leds[i].set_clamped(r, g, b);
    }
}

/// `rsp440` (registered as `a9k-rsp440-tr`): the RSP440-SE / -SE-2 panels.
#[derive(Default)]
pub struct RSP440 {
    disk0: Option<Box<dyn Fake + Send>>,
    disk1: Option<Box<dyn Fake + Send>>,
    // (gps, sync, maj, min, sso) index lists per card type.
    se: PanelLabels,
    se2: PanelLabels,
}

#[derive(Default)]
struct PanelLabels {
    gps: Vec<usize>,
    sync: Vec<usize>,
    maj: Vec<usize>,
    min: Vec<usize>,
    sso: Vec<usize>,
}

impl PanelLabels {
    fn capture(c: &Chassi, ty: &str) -> Self {
        PanelLabels {
            gps: c.leds_with_label_on_type(ty, "gps"),
            sync: c.leds_with_label_on_type(ty, "sync"),
            maj: c.leds_with_label_on_type(ty, "maj"),
            min: c.leds_with_label_on_type(ty, "min"),
            sso: c.leds_with_label_on_type(ty, "sso"),
        }
    }
}

impl Pattern for RSP440 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let d0 = self
            .disk0
            .as_mut()
            .map(|f| f.trig(info.secs))
            .unwrap_or(0.0);
        let d1 = self
            .disk1
            .as_mut()
            .map(|f| f.trig(info.secs))
            .unwrap_or(0.0);

        for labels in [&self.se, &self.se2] {
            set_all(c, &labels.gps, 1.0, 0.0, 0.0);
            set_all(c, &labels.sync, 1.0, 0.0, 0.0);
            set_all(c, &labels.maj, 0.0, 1.0, 0.0);
            set_all(c, &labels.min, 1.0, 0.5, 0.0);
        }
        let (se_sso, se2_sso) = (self.se.sso.clone(), self.se2.sso.clone());
        set_all(c, &se_sso, 0.0, 0.0, d0);
        set_all(c, &se2_sso, 0.0, 0.0, d1);
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "a9k-rsp440-tr",
            category: "misc",
        }
    }
    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        self.disk0 = Some(ri(40.0, 1200.0, 0.1, 6.5, ctx));
        self.disk1 = Some(ri(40.0, 1200.0, 0.1, 6.5, ctx));
        self.se = PanelLabels::capture(c, "A9K-RSP440-SE");
        self.se2 = PanelLabels::capture(c, "A9K-RSP440-SE-2");
    }
}

/// `sup720`: the Cisco 7609 supervisor panel.
#[derive(Default)]
pub struct SUP720 {
    disk0: Option<Box<dyn Fake + Send>>,
    disk1: Option<Box<dyn Fake + Send>>,
    port0: Option<Box<dyn Fake + Send>>,
    port1: Option<Box<dyn Fake + Send>>,
    system: Vec<usize>,
    active: Vec<usize>,
    mgmt: Vec<usize>,
    disk0_led: Vec<usize>,
    disk1_led: Vec<usize>,
    p1: Vec<usize>,
    p2: Vec<usize>,
}

impl Pattern for SUP720 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let d0 = self
            .disk0
            .as_mut()
            .map(|f| f.trig(info.secs))
            .unwrap_or(0.0);
        let d1 = self
            .disk1
            .as_mut()
            .map(|f| f.trig(info.secs))
            .unwrap_or(0.0);
        let p0v = utils::invert(
            self.port0
                .as_mut()
                .map(|f| f.trig(info.secs))
                .unwrap_or(0.0),
        );
        let p1v = utils::invert(
            self.port1
                .as_mut()
                .map(|f| f.trig(info.secs))
                .unwrap_or(0.0),
        );

        set_all(c, &self.system, 0.2, 1.0, 0.0);
        set_all(c, &self.active, 0.2, 1.0, 0.0);
        set_all(c, &self.mgmt, 1.0, 0.0, 0.0);
        set_all(c, &self.disk0_led, 0.0, d0, 0.0);
        set_all(c, &self.disk1_led, 0.0, d1, 0.0);
        set_all(c, &self.p1, 0.7 * p0v, 0.5 * p0v, 0.0);
        set_all(c, &self.p2, 0.7 * p1v, 0.5 * p1v, 0.0);
    }
    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "sup720",
            category: "misc",
        }
    }
    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        self.disk0 = Some(ri(40.0, 12000.0, 0.1, 6.5, ctx));
        self.disk1 = Some(ri(40.0, 12000.0, 0.1, 6.5, ctx));
        self.port0 = Some(ri(0.3, 12.0, 0.07, 6.5, ctx));
        self.port1 = Some(ri(0.2, 7.0, 0.07, 12.0, ctx));
        let ty = "sup720";
        self.system = c.leds_with_label_on_type(ty, "system");
        self.active = c.leds_with_label_on_type(ty, "active");
        self.mgmt = c.leds_with_label_on_type(ty, "mgmt");
        self.disk0_led = c.leds_with_label_on_type(ty, "disk0");
        self.disk1_led = c.leds_with_label_on_type(ty, "disk1");
        self.p1 = c.leds_with_label_on_type(ty, "p1");
        self.p2 = c.leds_with_label_on_type(ty, "p2");
    }
}
