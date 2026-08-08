//! slisko firmware for the bong69 / WT32-ETH01 board (ESP32, esp-idf std).
//!
//! Boots from the shared baked `config` crate (chassis, active patterns, ledinfo
//! output map, shaper), brings up Ethernet (LAN8720) + SNTP, and runs the render
//! loop driving either WS281x RMT channels or APA102 SPI chains off the
//! monotonic esp_timer clock. The Controller is shared (Arc<Mutex>) so the HTTP
//! control surface and DDP sink can drive it.

mod apa102;
mod board;
mod buttons;
mod ddp;
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

use engine::chassi::Chassi;
use engine::controller::Controller;
use engine::output::StrandMap;
use engine::traffic::{Shaper, ShaperConfig};

use apa102::Apa102Output;
use config as cfg;
use config::{LedDriver, LedOutput};
use output::Ws281xOutput;

const FPS: u32 = 60;

type Shared = Arc<Mutex<Controller>>;

enum PhysicalOutputs<'d> {
    Ws281x(Ws281xOutput<'d>),
    Apa102(Vec<Apa102Output<'d>>),
}

impl PhysicalOutputs<'_> {
    fn write(&mut self, leds: &[engine::pixel::Pixel]) -> Result<(), EspError> {
        match self {
            Self::Ws281x(output) => output.write(leds),
            Self::Apa102(outputs) => {
                for output in outputs {
                    output.write(leds)?;
                }
                Ok(())
            }
        }
    }
}

fn main() -> Result<(), EspError> {
    link_patches();
    EspLogger::initialize_default();
    info!("firmware booting");

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let pins = peripherals.pins;

    // --- LED outputs: degrade the 8 board data pins, then map the baked ledinfo
    // outputs onto them. (Partial move leaves the Ethernet pins available.) ---
    let mut led_slots: [(u8, Option<AnyOutputPin<'static>>); 8] = [
        (1, Some(pins.gpio1.degrade_output())),
        (2, Some(pins.gpio2.degrade_output())),
        (3, Some(pins.gpio3.degrade_output())),
        (4, Some(pins.gpio4.degrade_output())),
        (5, Some(pins.gpio5.degrade_output())),
        (12, Some(pins.gpio12.degrade_output())),
        (14, Some(pins.gpio14.degrade_output())),
        (15, Some(pins.gpio15.degrade_output())),
    ];
    let mut leds = match cfg::LED_DRIVER {
        LedDriver::Ws281x(kind) => {
            let outputs = map_ws281x_outputs(&mut led_slots);
            info!(
                "ledinfo: {:?}, {} RMT output(s) mapped",
                kind,
                outputs.len()
            );
            PhysicalOutputs::Ws281x(Ws281xOutput::new(outputs, kind)?)
        }
        LedDriver::Apa102(options) => {
            let mut outputs = Vec::new();
            let mut spi2 = Some(peripherals.spi2);
            let mut spi3 = Some(peripherals.spi3);
            for (index, output) in cfg::LED_OUTPUTS.iter().enumerate() {
                let LedOutput::Apa102 {
                    clock,
                    data,
                    start,
                    end,
                } = *output
                else {
                    warn!("ledinfo: ignoring WS281x mapping for APA102 driver");
                    continue;
                };
                let Some(clock_pin) = take_led_pin(&mut led_slots, clock) else {
                    continue;
                };
                let Some(data_pin) = take_led_pin(&mut led_slots, data) else {
                    continue;
                };
                let output = match index {
                    0 => Apa102Output::new(
                        spi2.take().expect("baker limits APA102 to SPI2 and SPI3"),
                        clock_pin,
                        data_pin,
                        start..end,
                        options,
                    )?,
                    1 => Apa102Output::new(
                        spi3.take().expect("baker limits APA102 to SPI2 and SPI3"),
                        clock_pin,
                        data_pin,
                        start..end,
                        options,
                    )?,
                    _ => unreachable!("baker limits APA102 to two chains"),
                };
                outputs.push(output);
            }
            info!(
                "ledinfo: APA102 {:?}, {} SPI output(s) mapped",
                options,
                outputs.len()
            );
            PhysicalOutputs::Apa102(outputs)
        }
    };

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
    let strand_map = StrandMap::new(&chassi, cfg::OUTPUT_MAPPING, cfg::LED_COUNT)
        .expect("baked output mapping must be valid");
    let seed = ((unsafe { esp_random() } as u64) << 32) | unsafe { esp_random() } as u64;
    let ctrl: Shared = Arc::new(Mutex::new(Controller::new(
        chassi,
        Shaper::new(shaper_config()),
        seed,
    )));
    {
        let mut c = ctrl.lock().unwrap();
        for &name in cfg::ACTIVE_PATTERNS {
            c.enable(name);
        }
        info!(
            "engine up: {} logical leds, {} output leds, {} active patterns; free heap = {} bytes",
            c.leds().len(),
            cfg::LED_COUNT,
            cfg::ACTIVE_PATTERNS.len(),
            unsafe { esp_idf_svc::sys::esp_get_free_heap_size() }
        );
    }

    // --- DDP sink (external override of internal patterns) ---
    let ddp_state = ddp::DdpState::new(cfg::LED_COUNT);
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
    let mut mapped_leds = Vec::with_capacity(cfg::LED_COUNT);
    loop {
        let elapsed_us = (unsafe { esp_timer_get_time() } - start_us) as u64;

        // Once a second, refresh the shaper hour from SNTP (if synced).
        if let Some(ts) = &timesync
            && ts.synced()
        {
            if !sntp_logged {
                info!("sntp synced; hour-of-day = {:.2}", time::hour_of_day());
                sntp_logged = true;
            }
            if let Ok(mut c) = ctrl.lock() {
                c.set_hour(time::hour_of_day());
            }
        }

        {
            let mut c = ctrl.lock().unwrap();
            if ddp_state.active() {
                // External source overrides internal patterns.
                ddp_state.apply(&strand_map, c.leds_mut());
            } else {
                c.tick_micros(elapsed_us);
            }
            strand_map.copy_pixels(c.leds(), &mut mapped_leds);
            leds.write(&mapped_leds)?;
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

/// Pair each baked WS281x data pin with its board pin and pixel range.
fn map_ws281x_outputs(
    by_gpio: &mut [(u8, Option<AnyOutputPin<'static>>); 8],
) -> Vec<(AnyOutputPin<'static>, core::ops::Range<usize>)> {
    let mut outputs = Vec::new();
    for o in cfg::LED_OUTPUTS {
        let LedOutput::Ws281x { data, start, end } = *o else {
            warn!("ledinfo: ignoring APA102 mapping for WS281x driver");
            continue;
        };
        if let Some(pin) = take_led_pin(by_gpio, data) {
            outputs.push((pin, start..end));
        }
    }
    outputs
}

/// Resolve one GPIO against the LED-capable pins on this bong69 revision while
/// preserving the existing unavailable/reused warnings.
fn take_led_pin(
    by_gpio: &mut [(u8, Option<AnyOutputPin<'static>>); 8],
    gpio: u8,
) -> Option<AnyOutputPin<'static>> {
    match by_gpio.iter_mut().find(|(candidate, _)| *candidate == gpio) {
        Some((_, slot @ Some(_))) => slot.take(),
        Some((_, None)) => {
            warn!("ledinfo: GPIO{gpio} used more than once; skipping");
            None
        }
        None => {
            warn!("ledinfo: GPIO{gpio} is not a board LED output; skipping");
            None
        }
    }
}
