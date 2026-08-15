//! Ported from `patterns/pattern.go` (the `Pattern` interface + info/render types).

use alloc::boxed::Box;

use crate::chassi::Chassi;
use crate::faker::Rng;

/// Metadata for a pattern (mirrors `PatternInfo`). Only one pattern per
/// non-`"misc"` category is active at a time (see the controller).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatternInfo {
    pub name: &'static str,
    pub category: &'static str,
}

/// Per-frame timing handed to every pattern (mirrors `RenderInfo`).
///
/// The Go version carries `Start time.Time` and patterns call
/// `time.Since(Start).Seconds()`. Here we pass the elapsed seconds directly so
/// the same code runs off `esp_timer` on-device and a wall clock in the sim.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderInfo {
    /// Seconds since the controller started.
    pub secs: f32,
    /// Wrapping whole milliseconds since the controller started. Deadline and
    /// interval logic uses this so it remains precise after long uptimes.
    pub millis: u32,
    /// Wrapping virtual milliseconds for traffic fakers. This clock advances
    /// at the live time-of-day intensity and can be held at zero during POST.
    pub traffic_millis: u32,
    /// Monotonic frame counter.
    pub frame: i64,
}

/// Context handed to [`Pattern::bootstrap`]: a PRNG replacing Go's global
/// `math/rand`. Traffic shaping is applied continuously by the controller's
/// virtual traffic clock rather than being captured here.
pub struct BootstrapCtx<'a> {
    pub rng: &'a mut Rng,
}

/// A render pattern (mirrors the Go `Pattern` interface).
pub trait Pattern {
    /// Mutate the chassis pixels for this frame.
    fn render(&mut self, info: &RenderInfo, c: &mut Chassi);
    /// Static metadata.
    fn info(&self) -> PatternInfo;
    /// One-time setup; reads the (immutable) chassis to capture index lists and
    /// builds fakers using the provided context.
    fn bootstrap(&mut self, c: &Chassi, ctx: &mut BootstrapCtx);
}

/// Boxed, dynamically-dispatched pattern — what the controller stores in its
/// active list and registry.
pub type BoxedPattern = Box<dyn Pattern + Send>;
