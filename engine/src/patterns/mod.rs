//! Pattern implementations. Chassis-specific patterns live in their hardware
//! family module; chassis-agnostic patterns live directly in this directory.

pub mod asr9000;
pub mod blinkstyle;
pub mod cisco7609;

mod blackout;
mod colorcycler;
mod green_status;
mod lamp_test;
mod panel_helpers;
mod port_faker;
mod pride;
mod rainbow;
mod red_status;
mod snake;
mod r#static;
mod strobe;

#[cfg(test)]
mod test_support;

pub use asr9000::{A9K8TL, A9K40GE, Mapper, RSP440};
pub use blackout::Blackout;
pub use cisco7609::{Blink48Ports, SUP720, X6704};
pub use colorcycler::Colorcycler;
pub use green_status::GreenStatus;
pub use lamp_test::LampTest;
pub use pride::Pride;
pub use rainbow::Rainbow;
pub use red_status::RedStatus;
pub use snake::Snake;
pub use r#static::Static;
pub use strobe::Strobe;
