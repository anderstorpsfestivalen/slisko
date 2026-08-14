//! Active-low redundant PSU inputs and their internal-render override.

use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver, Pull};
use log::{info, warn};

use engine::controller::Controller;
use engine::{RedundantPowerMonitor, RedundantPowerState};

use crate::health::Health;

const DEBOUNCE_MS: u64 = 30;

pub struct RedundantPowerInputs {
    pins: [Option<PinDriver<'static, Input>>; 2],
    monitor: RedundantPowerMonitor,
    mgmt_led: usize,
}

impl RedundantPowerInputs {
    pub fn new(
        header_pins: &mut Vec<(u8, AnyInputPin<'static>)>,
        controller: &Controller,
        health: Health,
    ) -> Option<Self> {
        let config = config::REDUNDANT_POWER?;
        let mgmt_led = controller
            .chassi()
            .leds_with_label_on_type("sup720", "mgmt")
            .into_iter()
            .next()
            .expect("baker requires a sup720 mgmt LED for redundant power");
        let mut pins = [None, None];

        for (index, gpio) in config.gpios.into_iter().enumerate() {
            let Some(pos) = header_pins
                .iter()
                .position(|(candidate, _)| *candidate == gpio)
            else {
                let message =
                    format!("redundant power input GPIO{gpio} is not an available header pin");
                warn!("{message}; treating PSU {} as offline", index + 1);
                health.record_error(message);
                continue;
            };
            let (_, pin) = header_pins.remove(pos);
            match PinDriver::input(pin, Pull::Up) {
                Ok(pin) => {
                    info!(
                        "redundant power: PSU {} connection on GPIO{} (low=online, high=offline)",
                        index + 1,
                        gpio
                    );
                    pins[index] = Some(pin);
                }
                Err(error) => {
                    let message = format!(
                        "redundant power PSU {} GPIO{} init failed: {error:?}",
                        index + 1,
                        gpio
                    );
                    warn!("{message}; treating input as offline");
                    health.record_error(message);
                }
            }
        }

        info!("redundant power: initial state offline until inputs settle");
        Some(Self {
            pins,
            monitor: RedundantPowerMonitor::new(DEBOUNCE_MS),
            mgmt_led,
        })
    }

    pub fn poll(&mut self, now_ms: u64) {
        let low = self
            .pins
            .each_ref()
            .map(|pin| pin.as_ref().is_some_and(PinDriver::is_low));
        if let Some(state) = self.monitor.update(low, now_ms) {
            match state {
                RedundantPowerState::Healthy => {
                    info!("redundant power: both PSUs online; mgmt LED green")
                }
                RedundantPowerState::Degraded => {
                    warn!("redundant power: one PSU offline; mgmt LED red")
                }
                RedundantPowerState::Offline => {
                    warn!("redundant power: both PSUs offline; blackout active")
                }
            }
        }
    }

    pub fn apply(&self, leds: &mut [engine::pixel::Pixel]) {
        self.monitor.state().apply(leds, self.mgmt_led);
    }
}
