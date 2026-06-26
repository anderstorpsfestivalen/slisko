//! slisko firmware for the bong69 / WT32-ETH01 board (ESP32, esp-idf std).
//!
//! Boots from the baked `generated_config.rs` (chassis, active patterns, ledinfo
//! output map, shaper), brings up Ethernet (LAN8720) + SNTP, and runs the render
//! loop driving the WS281x RMT channels off the monotonic esp_timer clock. The
//! Controller is shared (Arc<Mutex>) so the HTTP control surface / DDP sink can
//! drive it (added in later passes).

mod board;
mod buttons;
// APA102/SPI transport for non-bong69 boards (selected by ledinfo type=APA102);
// not instantiated here since this board's outputs are clockless WS281x.
#[allow(dead_code)]
mod apa102;
mod ddp;
mod generated_config;
mod http;
mod net;
mod output;
mod time;

use std::sync::{Arc, Mutex};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{EspError, esp_random, esp_timer_get_time, link_patches};
use log::{info, warn};

use slisko_core::chassi::Chassi;
use slisko_core::controller::Controller;
use slisko_core::output::LedType;
use slisko_core::traffic::{Shaper, ShaperConfig};

use generated_config as cfg;
use output::Ws281xOutput;

const FPS: u32 = 60;

type Shared = Arc<Mutex<Controller>>;

fn main() -> Result<(), EspError> {
    link_patches();
    EspLogger::initialize_default();
    info!("slisko-fw booting");

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let pins = peripherals.pins;

    // --- LED outputs: degrade the 8 board data pins, then map the baked ledinfo
    // outputs onto them. (Partial move leaves the Ethernet pins available.) ---
    let led_slots: [(u8, Option<AnyOutputPin<'static>>); 8] = [
        (1, Some(pins.gpio1.degrade_output())),
        (2, Some(pins.gpio2.degrade_output())),
        (3, Some(pins.gpio3.degrade_output())),
        (4, Some(pins.gpio4.degrade_output())),
        (5, Some(pins.gpio5.degrade_output())),
        (12, Some(pins.gpio12.degrade_output())),
        (14, Some(pins.gpio14.degrade_output())),
        (15, Some(pins.gpio15.degrade_output())),
    ];
    let ledtype = LedType::parse(cfg::LED_TYPE).unwrap_or(LedType::Ws2812);
    let outputs = map_outputs(led_slots);
    info!(
        "ledinfo: type {} ({:?}), {} output(s) mapped",
        cfg::LED_TYPE,
        ledtype,
        outputs.len()
    );
    if !ledtype.is_clockless() {
        warn!(
            "LED type {} is clocked (APA102); SPI transport not yet wired",
            cfg::LED_TYPE
        );
    }
    let mut leds = Ws281xOutput::new(outputs, ledtype)?;

    // --- Ethernet (the board's only network path) ---
    let eth_pins = net::EthPins {
        mac: peripherals.mac,
        gpio0: pins.gpio0,
        gpio16: pins.gpio16,
        gpio18: pins.gpio18,
        gpio19: pins.gpio19,
        gpio21: pins.gpio21,
        gpio22: pins.gpio22,
        gpio23: pins.gpio23,
        gpio25: pins.gpio25,
        gpio26: pins.gpio26,
        gpio27: pins.gpio27,
    };
    let _eth_guard = match net::bring_up(eth_pins, sysloop) {
        Ok(g) => Some(g),
        Err(e) => {
            warn!("ethernet bring-up failed: {e:?}");
            None
        }
    };

    // --- SNTP (feeds the shaper's hour-of-day) ---
    let timesync = match time::TimeSync::start() {
        Ok(t) => Some(t),
        Err(e) => {
            warn!("sntp start failed: {e:?}");
            None
        }
    };

    // --- Engine (shared) from the baked chassis + shaper ---
    let chassi = Chassi::from_specs(cfg::CHASSIS);
    let seed = ((unsafe { esp_random() } as u64) << 32) | unsafe { esp_random() } as u64;
    let ctrl: Shared = Arc::new(Mutex::new(Controller::new(
        chassi,
        Shaper::new(shaper_config()),
        seed,
    )));
    let num_leds;
    {
        let mut c = ctrl.lock().unwrap();
        for &name in cfg::ACTIVE_PATTERNS {
            c.enable(name);
        }
        num_leds = c.leds().len();
        info!(
            "engine up: {} leds, {} active patterns; free heap = {} bytes",
            num_leds,
            cfg::ACTIVE_PATTERNS.len(),
            unsafe { esp_idf_svc::sys::esp_get_free_heap_size() }
        );
    }

    // --- DDP sink (external override of internal patterns) ---
    let ddp_state = ddp::DdpState::new(num_leds);
    ddp::spawn(ddp_state.clone());

    // --- Buttons (expansion-header pins) -> scene switching ---
    let header_pins: Vec<(u8, esp_idf_hal::gpio::AnyInputPin<'static>)> = vec![
        (17, pins.gpio17.degrade_input()),
        (32, pins.gpio32.degrade_input()),
        (33, pins.gpio33.degrade_input()),
        (34, pins.gpio34.degrade_input()),
        (35, pins.gpio35.degrade_input()),
        (36, pins.gpio36.degrade_input()),
    ];
    buttons::spawn(header_pins, ctrl.clone());

    // --- HTTP control server + mDNS (kept alive for the program) ---
    let _http_guard = match http::start(ctrl.clone(), ddp_state.clone()) {
        Ok(g) => Some(g),
        Err(e) => {
            warn!("http start failed: {e:?}");
            None
        }
    };

    // --- Render loop ---
    let start_us = unsafe { esp_timer_get_time() };
    let frame_ms = (1000 / FPS).max(1);
    let mut sntp_logged = false;
    loop {
        let now = (unsafe { esp_timer_get_time() } - start_us) as f32 / 1_000_000.0;

        // Once a second, refresh the shaper hour from SNTP (if synced).
        if let Some(ts) = &timesync {
            if ts.synced() {
                if !sntp_logged {
                    info!("sntp synced; hour-of-day = {:.2}", time::hour_of_day());
                    sntp_logged = true;
                }
                if let Ok(mut c) = ctrl.lock() {
                    c.set_hour(time::hour_of_day());
                }
            }
        }

        {
            let mut c = ctrl.lock().unwrap();
            if ddp_state.active() {
                // External source overrides internal patterns.
                ddp_state.apply(c.leds_mut());
            } else {
                c.tick(now);
            }
            leds.write(c.leds())?;
        }
        FreeRtos::delay_ms(frame_ms);
    }
}

/// Convert the baked `ShaperConfig` literal into the core type.
fn shaper_config() -> ShaperConfig {
    let s = &cfg::SHAPER;
    ShaperConfig {
        enabled: s.enabled,
        peak_start: s.peak_start,
        peak_end: s.peak_end,
        low_start: s.low_start,
        low_end: s.low_end,
        peak_factor: s.peak_factor,
        low_factor: s.low_factor,
    }
}

/// Pair each baked `LedOutput` (matched by GPIO number) with the corresponding
/// board pin and pixel range. Non-board GPIOs are skipped with a warning.
fn map_outputs(
    mut by_gpio: [(u8, Option<AnyOutputPin<'static>>); 8],
) -> Vec<(AnyOutputPin<'static>, core::ops::Range<usize>)> {
    let mut outputs = Vec::new();
    for o in cfg::LED_OUTPUTS {
        match by_gpio.iter_mut().find(|(g, _)| *g == o.gpio) {
            Some((_, slot @ Some(_))) => outputs.push((slot.take().unwrap(), o.start..o.end)),
            Some((_, None)) => warn!("ledinfo: GPIO{} used more than once; skipping", o.gpio),
            None => warn!(
                "ledinfo: GPIO{} is not a board LED output; skipping",
                o.gpio
            ),
        }
    }
    outputs
}
