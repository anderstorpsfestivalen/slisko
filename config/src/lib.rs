//! Shared configuration for every slisko runtime.
//!
//! The host-side `build.rs` uses `baker` to compile the selected TOML
//! configuration into Rust source in Cargo's `OUT_DIR`.
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LedOutput {
    pub gpio: u8,
    pub start: usize,
    pub end: usize,
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
        assert!(
            LED_OUTPUTS
                .iter()
                .all(|output| output.start < output.end && output.end <= LED_COUNT)
        );
    }
}
