//! GPIO buttons → scene switching (ported behavior from `pkg/gpio`).
//!
//! Polls the baked `BUTTONS` on a background thread; on a falling edge (active-
//! low) it clears the controller and enables that button's scene — same as the
//! RPI's GPIO handler. Buttons whose GPIO isn't one of the board's expansion
//! header pins are skipped (e.g. configs carried over from the RPI that use pins
//! the WT32-ETH01 reserves for Ethernet).

use std::sync::{Arc, Mutex};

use esp_idf_hal::gpio::{AnyInputPin, Input, PinDriver, Pull};
use log::{info, warn};

use slisko_core::controller::Controller;

use crate::generated_config as cfg;

type Shared = Arc<Mutex<Controller>>;

struct Btn {
    pin: PinDriver<'static, Input>,
    scene: &'static [&'static str],
    last_low: bool,
}

/// Spawn the button-poll thread. `header_pins` are the available expansion-header
/// `(gpio, pin)` pairs.
pub fn spawn(header_pins: Vec<(u8, AnyInputPin<'static>)>, ctrl: Shared) {
    std::thread::Builder::new()
        .name("buttons".into())
        .stack_size(4096)
        .spawn(move || run(header_pins, ctrl))
        .expect("spawn buttons thread");
}

fn run(mut header_pins: Vec<(u8, AnyInputPin<'static>)>, ctrl: Shared) {
    let mut btns: Vec<Btn> = Vec::new();
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
        // 34/35/36 are input-only (no internal pull) — they need an external one.
        let pull = if matches!(gpio, 34 | 35 | 36) {
            Pull::Floating
        } else {
            Pull::Up
        };
        match PinDriver::input(pin, pull) {
            Ok(d) => {
                info!("button: GPIO{} -> {:?}", gpio, b.scene);
                btns.push(Btn {
                    pin: d,
                    scene: b.scene,
                    last_low: false,
                });
            }
            Err(e) => warn!("button GPIO{} init failed: {e:?}", gpio),
        }
    }

    if btns.is_empty() {
        info!("buttons: none active");
        return;
    }

    loop {
        for b in &mut btns {
            let low = b.pin.is_low();
            if low && !b.last_low {
                // Falling edge = press: clear + enable the scene.
                if let Ok(mut c) = ctrl.lock() {
                    c.clear();
                    for &p in b.scene {
                        c.enable(p);
                    }
                }
                info!("button pressed -> {:?}", b.scene);
            }
            b.last_low = low;
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}
