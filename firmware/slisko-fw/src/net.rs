//! Ethernet (LAN8720 RMII) bring-up for the WT32-ETH01.
//!
//! Ethernet is the board's only network path. We power-enable the PHY (GPIO16),
//! start the RMII driver, and return an opaque guard the caller keeps alive for
//! the program's lifetime. We deliberately do NOT block on the DHCP lease so the
//! LED render loop runs even before the cable is up; DHCP completes in the
//! background.

use std::any::Any;

use esp_idf_hal::gpio::{
    AnyOutputPin, Gpio0, Gpio16, Gpio18, Gpio19, Gpio21, Gpio22, Gpio23, Gpio25, Gpio26, Gpio27,
    PinDriver,
};
use esp_idf_hal::mac::MAC;
use esp_idf_svc::eth::{EspEth, EthDriver, RmiiClockConfig, RmiiEthChipset};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::sys::EspError;
use log::info;

use crate::board::eth as ethpins;

/// The pins Ethernet needs (the RMII data pins are hardware-fixed on the ESP32
/// EMAC; MDC/MDIO/clock/power are the WT32-ETH01 wiring).
#[allow(clippy::too_many_arguments)]
pub struct EthPins {
    pub mac: MAC<'static>,
    pub gpio0: Gpio0<'static>,
    pub gpio16: Gpio16<'static>,
    pub gpio18: Gpio18<'static>,
    pub gpio19: Gpio19<'static>,
    pub gpio21: Gpio21<'static>,
    pub gpio22: Gpio22<'static>,
    pub gpio23: Gpio23<'static>,
    pub gpio25: Gpio25<'static>,
    pub gpio26: Gpio26<'static>,
    pub gpio27: Gpio27<'static>,
}

/// Bring up Ethernet and return a guard (PHY power pin + started driver) that
/// must be kept alive.
pub fn bring_up(p: EthPins, sysloop: EspSystemEventLoop) -> Result<Box<dyn Any>, EspError> {
    // WT32-ETH01: drive the PHY power-enable high before init.
    let mut power = PinDriver::output(p.gpio16)?;
    power.set_high()?;

    let driver = EthDriver::new_rmii(
        p.mac,
        p.gpio25, // RXD0
        p.gpio26, // RXD1
        p.gpio27, // CRS_DV
        p.gpio23, // MDC
        p.gpio22, // TXD1
        p.gpio21, // TX_EN
        p.gpio19, // TXD0
        p.gpio18, // MDIO
        RmiiClockConfig::Input(p.gpio0),
        None::<AnyOutputPin>,
        RmiiEthChipset::LAN87XX, // LAN8720 is in the LAN87xx family
        Some(ethpins::PHY_ADDR),
        sysloop,
    )?;

    let mut eth = EspEth::wrap(driver)?;
    info!(
        "ethernet: starting (LAN8720 RMII, phy_addr={})",
        ethpins::PHY_ADDR
    );
    eth.start()?;
    info!("ethernet: started; DHCP proceeds in background");

    // Keep the power pin and driver alive for the program's lifetime.
    Ok(Box::new((power, eth)))
}
