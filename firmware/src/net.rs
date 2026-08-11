//! Supervised Ethernet (LAN8720 RMII) for the WT32-ETH01.
//!
//! ESP-IDF/lwIP owns normal DHCP retransmission and automatically restarts the
//! client after a link reconnect. This wrapper observes those state machines,
//! repairs a client that unexpectedly stops, and never blocks LED rendering
//! while the network is unavailable.

use esp_idf_hal::gpio::{
    Gpio0, Gpio16, Gpio18, Gpio19, Gpio21, Gpio22, Gpio23, Gpio25, Gpio26, Gpio27,
};
use esp_idf_hal::mac::MAC;
use esp_idf_svc::eth::{EspEth, EthDriver, RmiiClockConfig, RmiiEth, RmiiEthChipset};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::sys::{
    ESP_ERR_ESP_NETIF_DHCP_ALREADY_STARTED, ESP_OK, EspError, esp_netif_dhcp_status_t,
    esp_netif_dhcp_status_t_ESP_NETIF_DHCP_INIT, esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STARTED,
    esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STOPPED, esp_netif_dhcpc_get_status,
    esp_netif_dhcpc_start,
};
use log::{info, warn};

use crate::board::eth as ethpins;
use crate::health::Health;
use crate::recovery::ExponentialBackoff;

const RETRY_INITIAL_MS: u64 = 1_000;
const RETRY_MAX_MS: u64 = 30_000;

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

#[derive(Clone, Copy, Debug, Default)]
pub struct NetworkStatus {
    pub ip_up: bool,
    pub became_up: bool,
    pub lost_ip: bool,
}

pub struct NetworkManager {
    eth: EspEth<'static, RmiiEth>,
    health: Health,
    start_retry: ExponentialBackoff,
    dhcp_retry: ExponentialBackoff,
    last_started: bool,
    last_link: bool,
    last_ip_up: bool,
}

impl NetworkManager {
    pub fn new(
        p: EthPins,
        sysloop: EspSystemEventLoop,
        health: Health,
        now_ms: u64,
    ) -> Result<Self, EspError> {
        let driver = EthDriver::new_rmii(
            p.mac,
            p.gpio25,
            p.gpio26,
            p.gpio27,
            p.gpio23,
            p.gpio22,
            p.gpio21,
            p.gpio19,
            p.gpio18,
            RmiiClockConfig::Input(p.gpio0),
            // WT32-ETH01 GPIO16 is the LAN8720 power/reset line. Hand it to
            // ESP-IDF so lan87xx_reset_hw performs the required low/high reset
            // before reading the PHY identifier. Merely driving it high can
            // race PHY startup and produce `wrong chip OUI`.
            Some(p.gpio16),
            RmiiEthChipset::LAN87XX,
            Some(ethpins::PHY_ADDR),
            sysloop,
        )?;
        let eth = EspEth::wrap(driver)?;
        let mut manager = Self {
            eth,
            health,
            start_retry: ExponentialBackoff::new(RETRY_INITIAL_MS, RETRY_MAX_MS),
            dhcp_retry: ExponentialBackoff::new(RETRY_INITIAL_MS, RETRY_MAX_MS),
            last_started: false,
            last_link: false,
            last_ip_up: false,
        };

        info!(
            "ethernet: initializing (LAN8720 RMII, phy_addr={})",
            ethpins::PHY_ADDR
        );
        manager.try_start(now_ms);
        Ok(manager)
    }

    pub fn poll(&mut self, now_ms: u64) -> NetworkStatus {
        let started = self.eth.is_started().unwrap_or_else(|error| {
            self.health
                .record_error(format!("ethernet status failed: {error:?}"));
            false
        });
        if !started && self.start_retry.ready(now_ms) {
            self.try_start(now_ms);
        }

        let started = self.eth.is_started().unwrap_or(false);
        let link = started && self.eth.is_connected().unwrap_or(false);
        let ip_up = link && self.eth.is_up().unwrap_or(false);
        let ip = if ip_up {
            self.eth
                .netif()
                .get_ip_info()
                .ok()
                .map(|info| info.ip.to_string())
        } else {
            None
        };

        let dhcp = self.dhcp_state();
        if link
            && !ip_up
            && dhcp.is_some_and(|state| {
                state == esp_netif_dhcp_status_t_ESP_NETIF_DHCP_INIT
                    || state == esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STOPPED
            })
            && self.dhcp_retry.ready(now_ms)
        {
            self.repair_dhcp(now_ms);
        }
        if ip_up {
            self.dhcp_retry.reset();
        }

        if started != self.last_started {
            info!(
                "ethernet: driver {}",
                if started { "started" } else { "stopped" }
            );
        }
        if link != self.last_link {
            info!("ethernet: link {}", if link { "up" } else { "down" });
        }
        if ip_up != self.last_ip_up {
            if ip_up {
                info!(
                    "ethernet: DHCP lease acquired: {}",
                    ip.as_deref().unwrap_or("unknown")
                );
            } else {
                warn!("ethernet: IP lease lost; continuing offline");
            }
        }

        self.health.update(|health| {
            health.ethernet_driver = if started { "started" } else { "retrying" };
            health.ethernet_link = if link { "up" } else { "down" };
            health.dhcp = if ip_up {
                "leased"
            } else if !link {
                "waiting"
            } else {
                if dhcp == Some(esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STARTED) {
                    "negotiating"
                } else if dhcp.is_some() {
                    "repairing"
                } else {
                    "unknown"
                }
            };
            health.ip = ip;
        });

        let status = NetworkStatus {
            ip_up,
            became_up: ip_up && !self.last_ip_up,
            lost_ip: !ip_up && self.last_ip_up,
        };
        self.last_started = started;
        self.last_link = link;
        self.last_ip_up = ip_up;
        status
    }

    fn try_start(&mut self, now_ms: u64) {
        match self.eth.start() {
            Ok(()) => {
                self.start_retry.reset();
                self.health
                    .update(|health| health.ethernet_driver = "started");
            }
            Err(error) => {
                let delay = self.start_retry.fail(now_ms);
                warn!("ethernet: start failed ({error:?}); retrying in {delay} ms");
                self.health.update(|health| {
                    health.ethernet_driver = "retrying";
                    health.last_error = Some(format!("ethernet start failed: {error:?}"));
                });
            }
        }
    }

    fn dhcp_state(&self) -> Option<esp_netif_dhcp_status_t> {
        let mut status = esp_netif_dhcp_status_t_ESP_NETIF_DHCP_INIT;
        let result = unsafe { esp_netif_dhcpc_get_status(self.eth.netif().handle(), &mut status) };
        (result == ESP_OK).then_some(status)
    }

    fn repair_dhcp(&mut self, now_ms: u64) {
        let result = unsafe { esp_netif_dhcpc_start(self.eth.netif().handle()) };
        self.health.update(|health| {
            health.dhcp_repair_attempts = health.dhcp_repair_attempts.wrapping_add(1);
        });
        if result == ESP_OK || result == ESP_ERR_ESP_NETIF_DHCP_ALREADY_STARTED {
            info!("ethernet: DHCP client repaired/restarted");
            self.dhcp_retry.reset();
        } else {
            let delay = self.dhcp_retry.fail(now_ms);
            warn!("ethernet: DHCP repair failed ({result}); retrying in {delay} ms");
            self.health
                .record_error(format!("DHCP repair failed: {result}"));
        }
    }
}
