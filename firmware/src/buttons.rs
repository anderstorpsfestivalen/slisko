//! Active-low GPIO buttons, polled and debounced by the render task so GPIO
//! ownership cannot be stranded in a failed worker thread.

use std::sync::{Arc, Mutex};

use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver, Pull};
use log::{info, warn};

use config as cfg;
use config::ButtonAction;
use engine::controller::Controller;

use crate::health::{Health, ServiceState, lock_recover};
use crate::recovery::{ActiveLowDebouncer, ButtonEdge, ButtonSceneState};

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
    controller: Arc<Mutex<Controller>>,
    scenes: ButtonSceneState,
    health: Health,
}

impl Buttons {
    pub fn new(
        mut header_pins: Vec<(u8, AnyInputPin<'static>)>,
        controller: Arc<Mutex<Controller>>,
        health: Health,
    ) -> Self {
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
            controller,
            scenes: ButtonSceneState::default(),
            health,
        }
    }

    pub fn poll(&mut self, now_ms: u64) {
        if now_ms < self.next_poll_ms {
            return;
        }
        self.next_poll_ms = now_ms.saturating_add(POLL_INTERVAL_MS);

        let mut events = Vec::new();
        for (index, button) in self.btns.iter_mut().enumerate() {
            let low = button.pin.is_low();
            if let Some(edge) = button.debouncer.update(low, now_ms) {
                events.push((index, edge));
            }
        }

        for (index, edge) in events {
            match edge {
                ButtonEdge::Pressed => self.press(index),
                ButtonEdge::Released => self.release(index),
            }
        }
        self.health
            .update(|state| state.buttons = ServiceState::Running);
    }

    fn press(&mut self, index: usize) {
        let button = &self.btns[index];
        let name = button.name;
        let action = button.action;
        let patterns = button.patterns;
        info!("BUTTON {name} pressed: {action:?} -> {patterns:?}");

        let current_patterns = if action == ButtonAction::Momentary {
            lock_recover(&self.controller).active_pattern_names()
        } else {
            Vec::new()
        };
        self.scenes.press(index, action, &current_patterns);

        lock_recover(&self.controller).replace_patterns(patterns);
    }

    fn release(&mut self, index: usize) {
        let Some(restore_patterns) = self.scenes.release(index) else {
            return;
        };
        info!(
            "BUTTON {} released: restoring {:?}",
            self.btns[index].name, restore_patterns
        );
        lock_recover(&self.controller).replace_patterns(&restore_patterns);
    }
}
