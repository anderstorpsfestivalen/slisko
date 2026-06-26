//! Ported patterns (from the Go `patterns/` package). Each implements
//! [`crate::pattern::Pattern`]. The controller registry maps names → instances.

pub mod blinkstyle;

mod colorcycler;
mod globals;
mod linecards;
mod mapper;
mod panels;
mod r#static;
mod status;

pub use colorcycler::Colorcycler;
pub use globals::{Snake, Strobe};
pub use linecards::{A9K8TL, A9K40GE, Blink48Ports, X6704};
pub use mapper::Mapper;
pub use panels::{RSP440, SUP720};
pub use r#static::Static;
pub use status::{GreenStatus, RedStatus};
