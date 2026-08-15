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
pub enum ButtonAction {
    /// Replace the active pattern set when the button is pressed.
    Change,
    /// Temporarily replace the pattern set while the button is held, then
    /// restore the scene that was active before the press.
    Momentary,
    /// Latch the replacement scene until another button is pressed.
    Hold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    pub name: &'static str,
    pub gpio: u8,
    pub action: ButtonAction,
    pub patterns: &'static [&'static str],
}

/// Two active-low inputs that report whether redundant power supplies are
/// connected. A low GPIO level means that supply is online; the pull-up/high
/// state means it is offline or disconnected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedundantPower {
    pub gpios: [u8; 2],
}

#[allow(clippy::all)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

pub use generated::*;

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use engine::chassi::Chassi;
    use engine::output::StrandMap;
    use std::vec::Vec;

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

    #[test]
    fn redundant_power_post_order_covers_the_physical_strand_left_to_right() {
        if NAME != "Cisco 7609" {
            return;
        }
        assert!(REDUNDANT_POWER.is_some());

        let chassi = Chassi::from_specs(CHASSIS);
        let map = StrandMap::new(&chassi, OUTPUT_MAPPING, LED_COUNT).unwrap();
        let cards = (0..chassi.linecards.len()).collect::<Vec<_>>();
        let order = map.physical_order_by_cards(&chassi, &cards);
        let mut sorted = order.clone();
        sorted.sort_unstable();

        assert_eq!(order.len(), LED_COUNT);
        assert_eq!(sorted, (0..LED_COUNT).collect::<Vec<_>>());
        assert_eq!(&order[..49], &(1..50).collect::<Vec<_>>());
        assert_eq!(order[49], 0);

        let negotiating = map.physical_indices_for_logical(chassi.link_ports());
        let mgmt = chassi.leds_with_label_on_type("sup720", "mgmt")[0];
        let mgmt_physical = map.physical_indices_for_logical(&[mgmt])[0];
        assert_eq!(negotiating.len(), 115);
        assert!(!negotiating.contains(&mgmt_physical));
    }
}
