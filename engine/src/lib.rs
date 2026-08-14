//! Portable slisko render core.
//!
//! Ported from the Go engine (`pkg/pixel`, `pkg/utils`, `pkg/chassi`,
//! `patterns/`, `pkg/faker`, `pkg/traffic`). This crate is `no_std` + `alloc`
//! so it runs on the ESP32, and compiles for the native `host` runner. The
//! graphical `sim` observes its mapped output over DDP.
//!
//! `f32` is used throughout (the ESP32 has a single-precision FPU); the Go
//! engine uses `float64`, so expect host/Go parity to rounding tolerance.
#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

pub mod chassi;
pub mod color;
pub mod controller;
pub mod faker;
mod math;
pub mod output;
pub mod pattern;
pub mod patterns;
pub mod pixel;
pub mod redundant_power;
pub mod traffic;
pub mod utils;

pub use chassi::{Chassi, LineCard, LineCardSpec};
pub use controller::Controller;
pub use faker::Rng;
pub use output::{
    Apa102Encoder, Apa102Options, ColorOrder, MappingError, MappingSegment, StrandMap, Ws281xType,
};
pub use pattern::{BootstrapCtx, Pattern, PatternInfo, RenderInfo};
pub use pixel::Pixel;
pub use redundant_power::{RedundantPowerMonitor, RedundantPowerState};
pub use traffic::{Shaper, ShaperConfig};
