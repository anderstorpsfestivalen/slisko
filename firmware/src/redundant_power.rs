//! Active-low redundant PSU inputs and their internal-render override.

use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver, Pull};
use log::{info, warn};

use engine::controller::Controller;
use engine::output::StrandMap;
use engine::{
    PowerOnSequence, PowerSequencePhase, PowerSequenceStatus, RedundantPowerMonitor,
    RedundantPowerState,
};

use crate::health::Health;

const DEBOUNCE_MS: u64 = 30;

pub struct RedundantPowerInputs {
    gpios: [u8; 2],
    pins: [Option<PinDriver<'static, Input>>; 2],
    last_raw: [Option<bool>; 2],
    monitor: RedundantPowerMonitor,
    sequence: PowerOnSequence,
    last_phase: PowerSequencePhase,
    mgmt_led: usize,
}

impl RedundantPowerInputs {
    pub fn new(
        header_pins: &mut Vec<(u8, AnyInputPin<'static>)>,
        controller: &Controller,
        strand_map: &StrandMap,
        seed: u64,
        health: Health,
    ) -> Option<Self> {
        let config = config::REDUNDANT_POWER?;
        let mgmt_led = controller
            .chassi()
            .leds_with_label_on_type("sup720", "mgmt")
            .into_iter()
            .next()
            .expect("baker requires a sup720 mgmt LED for redundant power");
        let card_order = (0..controller.chassi().linecards.len()).collect::<Vec<_>>();
        let sweep_order = strand_map.physical_order_by_cards(controller.chassi(), &card_order);
        let negotiation_leds =
            strand_map.physical_indices_for_logical(controller.chassi().link_ports());
        let negotiation_led_count = negotiation_leds.len();
        assert_eq!(
            sweep_order.len(),
            strand_map.len(),
            "POST sweep must cover every physical LED"
        );
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

        let mut inputs = Self {
            gpios: config.gpios,
            pins,
            last_raw: [None; 2],
            monitor: RedundantPowerMonitor::new(DEBOUNCE_MS),
            sequence: PowerOnSequence::new(sweep_order, negotiation_leds, seed ^ 0x7609_504f_5354),
            last_phase: PowerSequencePhase::Initializing,
            mgmt_led,
        };
        inputs.last_raw = inputs.read_raw();
        inputs.log_raw_snapshot("boot", inputs.last_raw);
        info!(
            "GPIO DEBUG boot: debounced state starts {:?}; waiting {}ms for stable inputs",
            inputs.monitor.state(),
            DEBOUNCE_MS
        );
        info!(
            "redundant power: POST sweep covers {} physical LEDs left-to-right; {} link LEDs negotiate over 0-5s",
            strand_map.len(),
            negotiation_led_count
        );
        Some(inputs)
    }

    pub fn poll(&mut self, now_ms: u64) {
        let raw = self.read_raw();
        let mut changed = false;
        for (index, &current) in raw.iter().enumerate() {
            if current != self.last_raw[index] {
                info!(
                    "GPIO DEBUG edge @{now_ms}ms: PSU{} GPIO{} {} -> {}",
                    index + 1,
                    self.gpios[index],
                    raw_level(self.last_raw[index]),
                    raw_level(current)
                );
                changed = true;
            }
        }
        if changed {
            self.log_raw_snapshot("change", raw);
            self.last_raw = raw;
        }

        let low = raw.map(|level| level.unwrap_or(false));
        if let Some(state) = self.monitor.update(low, now_ms) {
            self.sequence.observe_power(state, now_ms);
            info!(
                "GPIO DEBUG debounced @{now_ms}ms: GPIO{}={} GPIO{}={} => {:?}",
                self.gpios[0],
                raw_level(raw[0]),
                self.gpios[1],
                raw_level(raw[1]),
                state
            );
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

    pub fn prepare_controller(
        &mut self,
        now_ms: u64,
        controller: &mut Controller,
    ) -> PowerSequenceStatus {
        let status = self.sequence.update(now_ms);
        if status.phase != self.last_phase {
            info!(
                "redundant power: POST phase {:?} -> {:?}",
                self.last_phase, status.phase
            );
            self.last_phase = status.phase;
        }
        if status.restart_traffic {
            info!("redundant power: restarting scene at zero traffic for 30s ramp");
            controller.restart_traffic();
        }
        controller.set_traffic_scale(status.traffic_scale);
        status
    }

    pub fn apply_logical(&self, status: PowerSequenceStatus, leds: &mut [engine::pixel::Pixel]) {
        if matches!(
            status.phase,
            PowerSequencePhase::Normal | PowerSequencePhase::Ramp
        ) {
            self.monitor.state().apply(leds, self.mgmt_led);
        }
    }

    pub fn apply_physical(&self, status: PowerSequenceStatus, leds: &mut [engine::pixel::Pixel]) {
        self.sequence.apply_physical(status, leds);
    }

    fn read_raw(&self) -> [Option<bool>; 2] {
        self.pins
            .each_ref()
            .map(|pin| pin.as_ref().map(PinDriver::is_low))
    }

    fn log_raw_snapshot(&self, reason: &str, raw: [Option<bool>; 2]) {
        let raw_state = RedundantPowerState::from_online(raw.map(|level| level.unwrap_or(false)));
        info!(
            "GPIO DEBUG {reason}: PSU1 GPIO{}={} PSU2 GPIO{}={} => raw-derived {:?}",
            self.gpios[0],
            raw_level(raw[0]),
            self.gpios[1],
            raw_level(raw[1]),
            raw_state
        );
    }
}

fn raw_level(low: Option<bool>) -> &'static str {
    match low {
        Some(true) => "LOW (0)",
        Some(false) => "HIGH (1)",
        None => "UNAVAILABLE",
    }
}
