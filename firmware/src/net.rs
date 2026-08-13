//! Ethernet-preferred network supervision for the WT32-ETH01.
//!
//! Ethernet is started immediately and remains the preferred lwIP route. When
//! it has no DHCP lease for 30 seconds, WiFi station mode cycles through the
//! configured credentials without blocking the render loop. As soon as
//! Ethernet has an IP again, WiFi is stopped. ESP-IDF owns DHCP retransmission;
//! this wrapper observes and repairs the underlying clients.

use esp_idf_hal::gpio::{
    Gpio0, Gpio16, Gpio18, Gpio19, Gpio21, Gpio22, Gpio23, Gpio25, Gpio26, Gpio27,
};
use esp_idf_hal::mac::MAC;
use esp_idf_hal::modem::Modem;
use esp_idf_svc::eth::{EspEth, EthDriver, RmiiClockConfig, RmiiEth, RmiiEthChipset};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::netif::{EspNetif, NetifConfiguration};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{
    ESP_ERR_ESP_NETIF_DHCP_ALREADY_STARTED, ESP_OK, EspError, esp_netif_dhcp_status_t,
    esp_netif_dhcp_status_t_ESP_NETIF_DHCP_INIT, esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STARTED,
    esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STOPPED, esp_netif_dhcpc_get_status,
    esp_netif_dhcpc_start,
};
use esp_idf_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, EspWifi};
use log::{info, warn};

use crate::board::eth as ethpins;
use crate::credentials::WIFI_NETWORKS;
use crate::health::Health;
use crate::network_policy::{
    FallbackCommand, FallbackPolicy, credentials_valid, next_credential, wifi_attempt_deadline,
};
use crate::recovery::ExponentialBackoff;

const RETRY_INITIAL_MS: u64 = 1_000;
const RETRY_MAX_MS: u64 = 30_000;
const WIFI_LIST_RETRY_MS: u64 = 5_000;
const ETHERNET_ROUTE_PRIORITY: u32 = 100;
const WIFI_ROUTE_PRIORITY: u32 = 50;

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

#[derive(Clone, Copy, Debug)]
enum WifiPhase {
    Stopped,
    Starting { credential: usize, deadline_ms: u64 },
    Connecting { credential: usize, deadline_ms: u64 },
    Connected { credential: usize },
    Retrying { credential: usize, retry_at_ms: u64 },
}

impl WifiPhase {
    fn credential(self) -> Option<usize> {
        match self {
            Self::Stopped => None,
            Self::Starting { credential, .. }
            | Self::Connecting { credential, .. }
            | Self::Connected { credential }
            | Self::Retrying { credential, .. } => Some(credential),
        }
    }
}

struct EthernetStatus {
    ip_up: bool,
    ip: Option<String>,
}

pub struct NetworkManager {
    eth: EspEth<'static, RmiiEth>,
    wifi: Option<EspWifi<'static>>,
    wifi_phase: WifiPhase,
    policy: FallbackPolicy,
    health: Health,
    start_retry: ExponentialBackoff,
    dhcp_retry: ExponentialBackoff,
    last_started: bool,
    last_link: bool,
    last_eth_ip_up: bool,
    last_network_ip_up: bool,
}

impl NetworkManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        p: EthPins,
        modem: Modem<'static>,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
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
            // before reading the PHY identifier.
            Some(p.gpio16),
            RmiiEthChipset::LAN87XX,
            Some(ethpins::PHY_ADDR),
            sysloop.clone(),
        )?;
        let mut eth_netif = NetifConfiguration::eth_default_client();
        eth_netif.route_priority = ETHERNET_ROUTE_PRIORITY;
        let eth = EspEth::wrap_all(driver, EspNetif::new_with_conf(&eth_netif)?)?;

        let wifi = match construct_wifi(modem, sysloop, nvs) {
            Ok(wifi) => Some(wifi),
            Err(error) => {
                warn!("wifi: construction failed ({error:?}); Ethernet remains available");
                health.record_error(format!("WiFi construction failed: {error:?}"));
                None
            }
        };

        let mut manager = Self {
            eth,
            wifi,
            wifi_phase: WifiPhase::Stopped,
            policy: FallbackPolicy::new(now_ms),
            health,
            start_retry: ExponentialBackoff::new(RETRY_INITIAL_MS, RETRY_MAX_MS),
            dhcp_retry: ExponentialBackoff::new(RETRY_INITIAL_MS, RETRY_MAX_MS),
            last_started: false,
            last_link: false,
            last_eth_ip_up: false,
            last_network_ip_up: false,
        };

        info!(
            "ethernet: initializing (LAN8720 RMII, phy_addr={}, route_priority={ETHERNET_ROUTE_PRIORITY})",
            ethpins::PHY_ADDR
        );
        info!("wifi: fallback route priority = {WIFI_ROUTE_PRIORITY}");
        manager.try_start_ethernet(now_ms);
        manager.update_wifi_health(false, None);
        Ok(manager)
    }

    pub fn poll(&mut self, now_ms: u64) -> NetworkStatus {
        let ethernet = self.poll_ethernet(now_ms);

        match self.policy.poll(now_ms, ethernet.ip_up) {
            FallbackCommand::StartWifi => self.start_wifi(now_ms, 0),
            FallbackCommand::StopWifi => self.stop_wifi(),
            FallbackCommand::None => {}
        }
        if self.policy.wifi_requested() {
            self.poll_wifi(now_ms);
        }

        let wifi_ip = self.wifi_ip();
        let wifi_ip_up = wifi_ip.is_some();
        let (active, active_ip) = if ethernet.ip_up {
            ("ethernet", ethernet.ip.clone())
        } else if wifi_ip_up {
            ("wifi", wifi_ip.clone())
        } else {
            ("none", None)
        };
        let ip_up = active_ip.is_some();

        self.update_wifi_health(ethernet.ip_up, wifi_ip);
        self.health.update(|health| {
            health.network_active = active;
            health.network_ip = active_ip;
        });

        let status = NetworkStatus {
            ip_up,
            became_up: ip_up && !self.last_network_ip_up,
            lost_ip: !ip_up && self.last_network_ip_up,
        };
        self.last_network_ip_up = ip_up;
        status
    }

    fn poll_ethernet(&mut self, now_ms: u64) -> EthernetStatus {
        let started = self.eth.is_started().unwrap_or_else(|error| {
            self.health
                .record_error(format!("ethernet status failed: {error:?}"));
            false
        });
        if !started && self.start_retry.ready(now_ms) {
            self.try_start_ethernet(now_ms);
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
        if ip_up != self.last_eth_ip_up {
            if ip_up {
                info!(
                    "ethernet: DHCP lease acquired: {}",
                    ip.as_deref().unwrap_or("unknown")
                );
            } else {
                warn!("ethernet: IP lease lost; WiFi fallback in 30 seconds");
            }
        }

        self.health.update(|health| {
            health.ethernet_driver = if started { "started" } else { "retrying" };
            health.ethernet_link = if link { "up" } else { "down" };
            health.dhcp = if ip_up {
                "leased"
            } else if !link {
                "waiting"
            } else if dhcp == Some(esp_netif_dhcp_status_t_ESP_NETIF_DHCP_STARTED) {
                "negotiating"
            } else if dhcp.is_some() {
                "repairing"
            } else {
                "unknown"
            };
            health.ip = ip.clone();
        });

        self.last_started = started;
        self.last_link = link;
        self.last_eth_ip_up = ip_up;
        EthernetStatus { ip_up, ip }
    }

    fn try_start_ethernet(&mut self, now_ms: u64) {
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

    fn start_wifi(&mut self, now_ms: u64, credential: usize) {
        if WIFI_NETWORKS.is_empty() {
            warn!("wifi: no credentials configured; fallback unavailable");
            self.health
                .record_error("WiFi fallback has no configured credentials");
            self.wifi_phase = WifiPhase::Stopped;
            return;
        }
        let Some(wifi) = self.wifi.as_mut() else {
            self.wifi_phase = WifiPhase::Stopped;
            return;
        };

        if wifi.is_started().unwrap_or(false) {
            self.begin_wifi_attempt(now_ms, credential);
            return;
        }

        match wifi.start() {
            Ok(()) => {
                info!("wifi: station radio starting");
                self.wifi_phase = WifiPhase::Starting {
                    credential,
                    deadline_ms: wifi_attempt_deadline(now_ms),
                };
            }
            Err(error) => {
                warn!("wifi: start failed ({error:?}); retrying");
                self.health
                    .record_error(format!("WiFi start failed: {error:?}"));
                self.wifi_phase = WifiPhase::Retrying {
                    credential,
                    retry_at_ms: now_ms.saturating_add(WIFI_LIST_RETRY_MS),
                };
            }
        }
    }

    fn poll_wifi(&mut self, now_ms: u64) {
        match self.wifi_phase {
            WifiPhase::Stopped => {}
            WifiPhase::Starting {
                credential,
                deadline_ms,
            } => {
                let started = self
                    .wifi
                    .as_ref()
                    .is_some_and(|wifi| wifi.is_started().unwrap_or(false));
                if started {
                    self.begin_wifi_attempt(now_ms, credential);
                } else if now_ms >= deadline_ms {
                    warn!("wifi: radio start timed out; retrying");
                    self.stop_wifi_driver();
                    self.wifi_phase = WifiPhase::Retrying {
                        credential,
                        retry_at_ms: now_ms.saturating_add(WIFI_LIST_RETRY_MS),
                    };
                }
            }
            WifiPhase::Connecting {
                credential,
                deadline_ms,
            } => {
                if let Some(ip) = self.wifi_ip() {
                    info!(
                        "wifi: connected to {:?}, ip = {}",
                        WIFI_NETWORKS[credential].0,
                        ip
                    );
                    self.wifi_phase = WifiPhase::Connected { credential };
                } else if now_ms >= deadline_ms {
                    warn!(
                        "wifi: {:?} did not connect within 15 seconds",
                        WIFI_NETWORKS[credential].0
                    );
                    self.disconnect_wifi_driver();
                    self.schedule_after_attempt(now_ms, credential);
                }
            }
            WifiPhase::Connected { credential } => {
                if self.wifi_ip().is_none() {
                    warn!("wifi: link lost; cycling configured networks");
                    self.disconnect_wifi_driver();
                    self.wifi_phase = WifiPhase::Retrying {
                        credential: 0,
                        retry_at_ms: now_ms.saturating_add(1_000),
                    };
                } else {
                    self.wifi_phase = WifiPhase::Connected { credential };
                }
            }
            WifiPhase::Retrying {
                credential,
                retry_at_ms,
            } => {
                if now_ms >= retry_at_ms {
                    self.start_wifi(now_ms, credential);
                }
            }
        }
    }

    fn begin_wifi_attempt(&mut self, now_ms: u64, credential: usize) {
        let (ssid, password) = WIFI_NETWORKS[credential];
        let Some(configuration) = wifi_configuration(ssid, password) else {
            warn!("wifi: invalid credential at index {credential}; skipping");
            self.health
                .record_error(format!("Invalid WiFi credential at index {credential}"));
            self.schedule_after_attempt(now_ms, credential);
            return;
        };

        self.health.update(|health| {
            health.wifi_attempts = health.wifi_attempts.wrapping_add(1);
        });
        info!("wifi: trying {ssid:?}");
        let result = self
            .wifi
            .as_mut()
            .expect("WiFi attempt requires a constructed driver")
            .set_configuration(&configuration)
            .and_then(|()| {
                self.wifi
                    .as_mut()
                    .expect("WiFi driver remains constructed")
                    .connect()
            });
        match result {
            Ok(()) => {
                self.wifi_phase = WifiPhase::Connecting {
                    credential,
                    deadline_ms: wifi_attempt_deadline(now_ms),
                };
            }
            Err(error) => {
                warn!("wifi: {ssid:?} attempt failed ({error:?})");
                self.health
                    .record_error(format!("WiFi {ssid:?} attempt failed: {error:?}"));
                self.schedule_after_attempt(now_ms, credential);
            }
        }
    }

    fn schedule_after_attempt(&mut self, now_ms: u64, credential: usize) {
        let (credential, retry_at_ms) = next_credential(credential, WIFI_NETWORKS.len(), now_ms);
        self.wifi_phase = WifiPhase::Retrying {
            credential,
            retry_at_ms,
        };
    }

    fn stop_wifi(&mut self) {
        if !matches!(self.wifi_phase, WifiPhase::Stopped) {
            info!("wifi: Ethernet DHCP recovered; stopping fallback radio");
        }
        self.stop_wifi_driver();
        self.wifi_phase = WifiPhase::Stopped;
    }

    fn disconnect_wifi_driver(&mut self) {
        if let Some(wifi) = self.wifi.as_mut()
            // Calling disconnect while association is still pending cancels
            // that attempt as well as an established connection.
            && wifi.is_started().unwrap_or(false)
            && let Err(error) = wifi.disconnect()
        {
            warn!("wifi: disconnect failed ({error:?})");
        }
    }

    fn stop_wifi_driver(&mut self) {
        self.disconnect_wifi_driver();
        if let Some(wifi) = self.wifi.as_mut()
            && wifi.is_started().unwrap_or(false)
            && let Err(error) = wifi.stop()
        {
            warn!("wifi: stop failed ({error:?})");
            self.health
                .record_error(format!("WiFi stop failed: {error:?}"));
        }
    }

    fn wifi_ip(&self) -> Option<String> {
        let wifi = self.wifi.as_ref()?;
        if !wifi.sta_netif().is_up().unwrap_or(false) {
            return None;
        }
        wifi.sta_netif()
            .get_ip_info()
            .ok()
            .map(|info| info.ip.to_string())
    }

    fn update_wifi_health(&self, ethernet_ip_up: bool, wifi_ip: Option<String>) {
        let requested = self.policy.wifi_requested();
        let phase = self.wifi_phase;
        let ssid = phase
            .credential()
            .and_then(|index| WIFI_NETWORKS.get(index))
            .map(|(ssid, _)| (*ssid).to_owned());
        let state = if ethernet_ip_up {
            "disabled"
        } else if !requested {
            "waiting"
        } else {
            match phase {
                WifiPhase::Stopped => "retrying",
                WifiPhase::Starting { .. } | WifiPhase::Connecting { .. } => "connecting",
                WifiPhase::Connected { .. } => "connected",
                WifiPhase::Retrying { .. } => "retrying",
            }
        };
        self.health.update(|health| {
            health.wifi_state = state;
            health.wifi_ssid = ssid;
            health.wifi_ip = wifi_ip;
        });
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

fn construct_wifi(
    modem: Modem<'static>,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Result<EspWifi<'static>, EspError> {
    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs))?;
    let mut configuration = NetifConfiguration::wifi_default_client();
    configuration.key = "WIFI_STA_FALLBACK".try_into().unwrap();
    configuration.description = "wifi-fb".try_into().unwrap();
    configuration.route_priority = WIFI_ROUTE_PRIORITY;
    let old_netif = wifi.swap_netif_sta(EspNetif::new_with_conf(&configuration)?)?;
    drop(old_netif);
    Ok(wifi)
}

fn wifi_configuration(ssid: &str, password: &str) -> Option<Configuration> {
    if !credentials_valid(ssid, password) {
        return None;
    }
    Some(Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().ok()?,
        password: password.try_into().ok()?,
        auth_method: if password.is_empty() {
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        },
        ..Default::default()
    }))
}
