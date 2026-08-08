//! bong69 / WT32-ETH01 board facts (source: github.com/bobko69/8PortLEDDistro).
//!
//! Centralizes the physical pin map so the rest of the firmware refers to roles,
//! not magic numbers.
// Some entries are referenced only as later subsystems (Ethernet, buttons) land.
#![allow(dead_code)]

/// The 8 LED data-output GPIOs, in port order (LED1..LED8). Each is driven by
/// the RMT channel of the same index on the classic ESP32 (8 channels total).
pub const LED_GPIOS: [u8; 8] = [1, 2, 3, 4, 5, 12, 14, 15];

/// Ethernet (LAN8720 RMII) — WT32-ETH01 wiring.
pub mod eth {
    /// RMII SMI management clock / data.
    pub const MDC_GPIO: u8 = 23;
    pub const MDIO_GPIO: u8 = 18;
    /// PHY address on the SMI bus.
    pub const PHY_ADDR: u32 = 1;
    /// 50 MHz RMII reference clock arrives on GPIO0 (input mode).
    pub const RMII_CLK_GPIO: u8 = 0;
    /// PHY power-enable; must be driven high before the PHY responds.
    pub const PHY_POWER_GPIO: u8 = 16;
}

/// Expansion-header GPIOs available for buttons / sensors. 34/35/36 are
/// input-only.
pub const BUTTON_GPIOS: [u8; 6] = [17, 32, 33, 34, 35, 36];
