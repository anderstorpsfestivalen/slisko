//! Ported from `pkg/controller/controller.go`.
//!
//! Owns the chassis and the active-pattern list, applies the category rules
//! (one pattern per non-`"misc"` category), and renders frames. The Go version
//! ran its own ticker + broker; here [`Controller::tick`] renders a single frame
//! given the elapsed seconds, so the host sim and the firmware task drive the
//! cadence.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::chassi::Chassi;
use crate::faker::Rng;
use crate::pattern::{BootstrapCtx, BoxedPattern, PatternInfo, RenderInfo};
use crate::patterns;
use crate::traffic::Shaper;

/// The registry of pattern names, in a stable order (mirrors the Go `pt` map).
pub const PATTERN_NAMES: &[&str] = &[
    "blink48ports",
    "greenstatus",
    "redstatus",
    "strobe",
    "sup720",
    "x6704",
    "colorcycler",
    "snake",
    "mapper",
    "a9k-8t-l",
    "a9k-40ge-l",
    "a9k-rsp440-tr",
    "static",
];

/// Construct a fresh (un-bootstrapped) pattern instance by name.
pub fn make_pattern(name: &str) -> Option<BoxedPattern> {
    use patterns::*;
    let p: BoxedPattern = match name {
        "blink48ports" => Box::new(Blink48Ports::default()),
        "greenstatus" => Box::new(GreenStatus),
        "redstatus" => Box::new(RedStatus),
        "strobe" => Box::new(Strobe),
        "sup720" => Box::new(SUP720::default()),
        "x6704" => Box::new(X6704::default()),
        "colorcycler" => Box::new(Colorcycler::default()),
        "snake" => Box::new(Snake),
        "mapper" => Box::new(Mapper),
        "a9k-8t-l" => Box::new(A9K8TL::default()),
        "a9k-40ge-l" => Box::new(A9K40GE::default()),
        "a9k-rsp440-tr" => Box::new(RSP440::default()),
        "static" => Box::new(Static),
        _ => return None,
    };
    Some(p)
}

pub struct Controller {
    chassi: Chassi,
    shaper: Shaper,
    rng: Rng,
    /// Fractional hour-of-day fed to the shaper (from SNTP on-device).
    hour: f32,
    frame: i64,
    active: Vec<BoxedPattern>,
}

impl Controller {
    pub fn new(chassi: Chassi, shaper: Shaper, seed: u64) -> Self {
        Controller {
            chassi,
            shaper,
            rng: Rng::new(seed),
            hour: 0.0,
            frame: 0,
            active: Vec::new(),
        }
    }

    /// Update the hour-of-day used by the traffic shaper (call when SNTP ticks).
    pub fn set_hour(&mut self, hour: f32) {
        self.hour = hour;
    }

    pub fn chassi(&self) -> &Chassi {
        &self.chassi
    }

    /// Read-only view of the rendered strand (for the output stage).
    pub fn leds(&self) -> &[crate::pixel::Pixel] {
        &self.chassi.leds
    }

    /// Mutable view of the strand — used by an external pixel source (DDP) to
    /// overwrite the buffer instead of running patterns.
    pub fn leds_mut(&mut self) -> &mut [crate::pixel::Pixel] {
        &mut self.chassi.leds
    }

    pub fn frame(&self) -> i64 {
        self.frame
    }

    /// Info for every registered pattern + whether it's currently active.
    pub fn pattern_list(&self) -> Vec<(PatternInfo, bool)> {
        PATTERN_NAMES
            .iter()
            .filter_map(|&n| make_pattern(n).map(|p| p.info()))
            .map(|info| (info, self.is_active(info.name)))
            .collect()
    }

    pub fn is_active(&self, name: &str) -> bool {
        self.active.iter().any(|p| p.info().name == name)
    }

    /// Enable a pattern by name (mirrors `EnablePattern`): no-op if already on;
    /// disables others in the same category unless that category is `"misc"`;
    /// bootstraps the fresh instance with a shaper-derived intensity.
    pub fn enable(&mut self, name: &str) {
        if self.is_active(name) {
            return;
        }
        let Some(mut pattern) = make_pattern(name) else {
            return;
        };
        let category = pattern.info().category;
        if category != "misc" {
            self.active.retain(|p| p.info().category != category);
        }

        let intensity = self.shaper.intensity(self.hour);
        let mut ctx = BootstrapCtx {
            rng: &mut self.rng,
            intensity,
        };
        pattern.bootstrap(&self.chassi, &mut ctx);
        self.active.push(pattern);
    }

    /// Disable a pattern by name and blank the strand (mirrors `DisablePattern`).
    pub fn disable(&mut self, name: &str) {
        let before = self.active.len();
        self.active.retain(|p| p.info().name != name);
        if self.active.len() != before {
            self.blank();
        }
    }

    /// Disable all patterns and blank the strand (mirrors `ClearPatterns`).
    pub fn clear(&mut self) {
        self.active.clear();
        self.blank();
    }

    fn blank(&mut self) {
        for p in &mut self.chassi.leds {
            p.set_color(0.0, 0.0, 0.0);
        }
    }

    /// Render one frame at elapsed `now` seconds. Globals render first so other
    /// patterns paint on top (mirrors the Go render order).
    pub fn tick(&mut self, now: f32) {
        let info = RenderInfo {
            secs: now,
            frame: self.frame,
        };
        // SAFETY-of-borrows: split the &mut self borrow — patterns need &mut
        // chassi while iterating &mut active. They are distinct fields.
        let (active, chassi) = (&mut self.active, &mut self.chassi);
        for p in active.iter_mut() {
            if p.info().category == "global" {
                p.render(&info, chassi);
            }
        }
        for p in active.iter_mut() {
            if p.info().category != "global" {
                p.render(&info, chassi);
            }
        }
        self.frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chassi::LineCardSpec;
    use crate::pixel::Position;
    use crate::traffic::Shaper;

    static POS: &[Position] = &[Position {
        x: 0.0,
        y: 0.0,
        size: 1.0,
    }; 4];
    static SPECS: &[LineCardSpec] = &[LineCardSpec {
        name: "C",
        image: "",
        active: true,
        positions: POS,
        link: &[0, 1, 2, 3],
        status: Some(0),
        labeled: &[],
    }];

    fn ctrl() -> Controller {
        Controller::new(Chassi::from_specs(SPECS), Shaper::default(), 1)
    }

    #[test]
    fn enable_is_idempotent_and_category_exclusive() {
        let mut c = ctrl();
        c.enable("colorcycler"); // global
        c.enable("colorcycler"); // no-op
        assert_eq!(c.active.len(), 1);
        c.enable("strobe"); // also global -> replaces colorcycler
        assert!(c.is_active("strobe") && !c.is_active("colorcycler"));
        assert_eq!(c.active.len(), 1);
    }

    #[test]
    fn misc_patterns_stack() {
        let mut c = ctrl();
        c.enable("x6704");
        c.enable("a9k-8t-l"); // both misc -> coexist
        assert_eq!(c.active.len(), 2);
        c.disable("x6704");
        assert!(!c.is_active("x6704") && c.is_active("a9k-8t-l"));
    }

    #[test]
    fn tick_runs_without_panicking() {
        let mut c = ctrl();
        c.enable("colorcycler");
        c.enable("greenstatus");
        for i in 0..30 {
            c.tick(i as f32 * 0.1);
        }
        assert_eq!(c.frame(), 30);
    }
}
