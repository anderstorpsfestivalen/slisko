//! GPIO buttons → scene switching, polled by the render task so button GPIO
//! ownership cannot be stranded in a failed worker thread.

use std::sync::{Arc, Mutex};

use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver, Pull};
use log::{info, warn};

use engine::controller::Controller;

use config as cfg;

use crate::health::{Health, ServiceState, lock_recover};

type Shared = Arc<Mutex<Controller>>;

struct Btn {
    pin: PinDriver<'static, Input>,
    scene: &'static [&'static str],
    last_low: bool,
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
            if b.scene.is_empty() {
                continue;
            }
            let Some(pos) = header_pins.iter().position(|(g, _)| *g == b.gpio) else {
                warn!(
                    "button GPIO{} is not an available header pin; skipping",
                    b.gpio
                );
                continue;
            };
            let (gpio, pin) = header_pins.remove(pos);
            let pull = if matches!(gpio, 34..=36) {
                Pull::Floating
            } else {
                Pull::Up
            };
            match PinDriver::input(pin, pull) {
                Ok(pin) => {
                    info!("button: GPIO{} -> {:?}", gpio, b.scene);
                    btns.push(Btn {
                        pin,
                        scene: b.scene,
                        last_low: false,
                    });
                }
                Err(error) => {
                    warn!("button GPIO{} init failed: {error:?}", gpio);
                    health.record_error(format!("button GPIO{gpio} init failed: {error:?}"));
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

    pub fn poll(&mut self, now_ms: u64, ctrl: &Shared) {
        if now_ms < self.next_poll_ms {
            return;
        }
        self.next_poll_ms = now_ms.saturating_add(30);

        for button in &mut self.btns {
            let low = button.pin.is_low();
            if low && !button.last_low {
                let mut controller = lock_recover(ctrl);
                controller.clear();
                for &pattern in button.scene {
                    controller.enable(pattern);
                }
                info!("button pressed -> {:?}", button.scene);
            }
            button.last_low = low;
        }
        self.health
            .update(|state| state.buttons = ServiceState::Running);
    }
}
