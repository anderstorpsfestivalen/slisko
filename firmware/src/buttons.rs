//! Active-low GPIO buttons, polled and debounced by the render task so GPIO
//! ownership cannot be stranded in a failed worker thread.

use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver, Pull};
use log::{debug, info, warn};

use config as cfg;
use config::ButtonAction;

use crate::health::{Health, ServiceState};
use crate::recovery::{ActiveLowDebouncer, ButtonEdge};

const POLL_INTERVAL_MS: u64 = 10;
const DEBOUNCE_MS: u64 = 30;

struct Btn {
    name: &'static str,
    pin: PinDriver<'static, Input>,
    action: ButtonAction,
    patterns: &'static [&'static str],
    debouncer: ActiveLowDebouncer,
}

pub struct Buttons {
    btns: Vec<Btn>,
    next_poll_ms: u64,
    health: Health,
}

impl Buttons {
    pub fn new(mut header_pins: Vec<(u8, AnyInputPin<'static>)>, health: Health) -> Self {
        let mut btns = Vec::new();
        for b in cfg::BUTTONS {
            let Some(pos) = header_pins.iter().position(|(g, _)| *g == b.gpio) else {
                warn!(
                    "button {}: GPIO{} is not an available header pin; skipping",
                    b.name, b.gpio
                );
                continue;
            };
            let (gpio, pin) = header_pins.remove(pos);
            let pull = if matches!(gpio, 34..=36) {
                warn!(
                    "button {}: GPIO{} has no internal pull-up; external pull-up to 3.3V required",
                    b.name, gpio
                );
                Pull::Floating
            } else {
                Pull::Up
            };
            match PinDriver::input(pin, pull) {
                Ok(pin) => {
                    info!(
                        "button: {} on GPIO{} (active-low, {:?}, patterns {:?})",
                        b.name, gpio, b.action, b.patterns
                    );
                    btns.push(Btn {
                        name: b.name,
                        pin,
                        action: b.action,
                        patterns: b.patterns,
                        debouncer: ActiveLowDebouncer::new(DEBOUNCE_MS),
                    });
                }
                Err(error) => {
                    warn!("button {} on GPIO{} init failed: {error:?}", b.name, gpio);
                    health.record_error(format!(
                        "button {} on GPIO{gpio} init failed: {error:?}",
                        b.name
                    ));
                }
            }
        }

        if btns.is_empty() {
            info!("buttons: none active");
        }
        health.update(|state| state.buttons = ServiceState::Running);
        Self {
            btns,
            next_poll_ms: 0,
            health,
        }
    }

    pub fn poll(&mut self, now_ms: u64) {
        if now_ms < self.next_poll_ms {
            return;
        }
        self.next_poll_ms = now_ms.saturating_add(POLL_INTERVAL_MS);

        for button in &mut self.btns {
            let low = button.pin.is_low();
            match button.debouncer.update(low, now_ms) {
                Some(ButtonEdge::Pressed) => {
                    info!("BUTTON {} pressed", button.name);
                    debug!(
                        "button action {:?}, patterns {:?}",
                        button.action, button.patterns
                    );
                }
                Some(ButtonEdge::Released) => {
                    debug!("BUTTON {} released", button.name);
                }
                None => {}
            }
        }
        self.health
            .update(|state| state.buttons = ServiceState::Running);
    }
}
