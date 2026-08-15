use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::faker::Fake;
use crate::pattern::{BootstrapCtx, LinkActivation, Pattern, PatternInfo, RenderInfo};
use crate::patterns::asr9000::blinkstyle::asr9000_mgmt_style;
use crate::patterns::blinkstyle::PortState;
use crate::patterns::panel_helpers::{random_interval, set_all};

/// The ASR9000 RSP440-SE and RSP440-SE-2 control panels.
#[derive(Default)]
pub struct RSP440 {
    disk0: Option<Box<dyn Fake + Send>>,
    disk1: Option<Box<dyn Fake + Send>>,
    ports: Vec<PortState>,
    se: PanelLabels,
    se2: PanelLabels,
}

#[derive(Default)]
struct PanelLabels {
    gps: Vec<usize>,
    sync: Vec<usize>,
    maj: Vec<usize>,
    min: Vec<usize>,
    ssd: Vec<usize>,
}

impl PanelLabels {
    fn capture(c: &Chassi, card_type: &str) -> Self {
        Self {
            gps: c.leds_with_label_on_type(card_type, "gps"),
            sync: c.leds_with_label_on_type(card_type, "sync"),
            maj: c.leds_with_label_on_type(card_type, "maj"),
            min: c.leds_with_label_on_type(card_type, "min"),
            ssd: c.leds_with_label_on_type(card_type, "ssd"),
        }
    }
}

impl Pattern for RSP440 {
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi) {
        let disk0 = self
            .disk0
            .as_mut()
            .map(|faker| faker.trig(info.millis))
            .unwrap_or(0.0);
        let disk1 = self
            .disk1
            .as_mut()
            .map(|faker| faker.trig(info.millis))
            .unwrap_or(0.0);

        for labels in [&self.se, &self.se2] {
            set_all(c, &labels.gps, 1.0, 0.0, 0.0);
            set_all(c, &labels.sync, 1.0, 0.0, 0.0);
            set_all(c, &labels.maj, 0.0, 1.0, 0.0);
            set_all(c, &labels.min, 1.0, 0.5, 0.0);
        }

        let (se_ssd, se2_ssd) = (self.se.ssd.clone(), self.se2.ssd.clone());
        set_all(c, &se_ssd, 0.0, 0.0, disk0);
        set_all(c, &se2_ssd, 0.0, 0.0, disk1);
        for port in &mut self.ports {
            port.render(&mut c.leds, info);
        }
    }

    fn info(&self) -> PatternInfo {
        PatternInfo {
            name: "a9k-rsp440-tr",
            category: "misc",
        }
    }

    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx) {
        self.disk0 = Some(random_interval(40.0, 1200.0, 0.1, 6.5, ctx));
        self.disk1 = Some(random_interval(40.0, 1200.0, 0.1, 6.5, ctx));
        self.se = PanelLabels::capture(c, "A9K-RSP440-SE");
        self.se2 = PanelLabels::capture(c, "A9K-RSP440-SE-2");
        let style = asr9000_mgmt_style();
        for card_type in ["A9K-RSP440-SE", "A9K-RSP440-SE-2"] {
            for index in c.link_indices_of_type(card_type) {
                self.ports.push(style.create_port(index, ctx));
            }
        }
    }

    fn restart_link_traffic(&mut self, now_ms: u32, activations: &[LinkActivation]) {
        for port in &mut self.ports {
            port.restart_link_traffic(now_ms, activations);
        }
    }
}
