//! Shared baked configuration for every slisko runtime.
//!
//! `generated.rs` is emitted by the Go `cmd/baker` command and remains checked
//! in so building the ESP32 firmware does not require Go.
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

mod generated;

pub use generated::*;

#[cfg(test)]
mod tests {
    use super::*;
    use slisko_core::chassi::Chassi;
    use slisko_core::output::StrandMap;

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
