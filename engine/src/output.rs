//! Hardware-independent LED mapping and device-specific wire encoding.
//!
//! Pattern and DDP colors are perceptual sRGB. Logical sRGB mapping lives in
//! [`mapping`], while physical conversion is isolated in [`ws281x`] and
//! [`apa102`]. Host DDP and the simulator use only the logical mapping API.

pub mod apa102;
pub mod mapping;
pub mod ws281x;

pub use apa102::{APA102_NEUTRAL_TEMPERATURE, Apa102Encoder, Apa102Options};
pub use mapping::{MappingError, MappingSegment, StrandMap};
pub use ws281x::{ColorOrder, Ws281xType, encode_ws281x};
