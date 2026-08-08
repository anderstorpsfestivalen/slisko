//! Shared configuration for every slisko runtime.
//!
//! The host-side `build.rs` uses `baker` to compile the selected TOML
//! configuration into Rust source in Cargo's `OUT_DIR`.
#![no_std]

use engine::output::{Apa102Options, Ws281xType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedDriver {
    Ws281x(Ws281xType),
    Apa102(Apa102Options),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedOutput {
    Ws281x {
        data: u8,
        start: usize,
        end: usize,
    },
    Apa102 {
        clock: u8,
        data: u8,
        start: usize,
        end: usize,
    },
}

impl LedOutput {
    pub const fn range(&self) -> (usize, usize) {
        match *self {
            Self::Ws281x { start, end, .. } | Self::Apa102 { start, end, .. } => (start, end),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    pub gpio: u8,
    pub scene: &'static [&'static str],
}

#[allow(clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub use generated::*;

#[cfg(test)]
mod tests {
    use super::*;
    use engine::chassi::Chassi;
    use engine::output::StrandMap;

    #[test]
    fn generated_mapping_and_output_ranges_are_valid() {
        let chassi = Chassi::from_specs(CHASSIS);
        let map = StrandMap::new(&chassi, OUTPUT_MAPPING, LED_COUNT).unwrap();
        assert_eq!(map.len(), LED_COUNT);
        assert!(LED_OUTPUTS.iter().all(|output| {
            let (start, end) = output.range();
            start < end && end <= LED_COUNT
        }));
    }
}
