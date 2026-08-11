//! slisko firmware for the bong69 / WT32-ETH01 board (ESP32, esp-idf std).
//!
//! LED rendering is the primary service. Ethernet, DHCP, SNTP, DDP, HTTP, and
//! buttons are supervised around it and may degrade without stopping patterns.

mod apa102;
mod board;
mod buttons;
mod ddp;
mod health;
mod http;
mod net;
mod output;
mod recovery;
mod time;

use std::sync::{Arc, Mutex};

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::{
    ESP_ERR_INVALID_STATE, ESP_OK, EspError, esp_random, esp_restart, esp_task_wdt_add,
    esp_task_wdt_config_t, esp_task_wdt_init, esp_task_wdt_reconfigure, esp_task_wdt_reset,
    esp_timer_get_time, link_patches,
};
use log::{error, info, warn};

use engine::chassi::Chassi;
use engine::controller::Controller;
use engine::output::StrandMap;
use engine::traffic::{Shaper, ShaperConfig};

use apa102::Apa102Output;
use config as cfg;
use config::{LedDriver, LedOutput};
use health::{Health, lock_recover};
use output::Ws281xOutput;
use recovery::FailureWindow;

const FPS: u32 = 60;
const SUPERVISOR_PERIOD_MS: u64 = 1_000;
const OUTPUT_RESTART_MS: u64 = 10_000;
const HARDWARE_RESTART_MS: u64 = 60_000;

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

fn main() {
    link_patches();
    EspLogger::initialize_default();
    info!("firmware booting");

    if let Err(error) = run() {
        error!("fatal firmware initialization error: {error:?}; rebooting in 5 seconds");
        FreeRtos::delay_ms(5_000);
        unsafe { esp_restart() };
    }
}

fn run() -> Result<(), EspError> {
    let boot_ms = monotonic_ms();
    let health = Health::new(boot_ms);
    configure_watchdog(&health);

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let pins = peripherals.pins;

    // --- Physical outputs ---
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

    // --- Engine first: patterns never wait for network or wall clock. ---
    let chassi = Chassi::from_specs(cfg::CHASSIS);
    let strand_map = StrandMap::new(&chassi, cfg::OUTPUT_MAPPING, cfg::LED_COUNT)
        .expect("baked output mapping must be valid");
    let seed = ((unsafe { esp_random() } as u64) << 32) | unsafe { esp_random() } as u64;
    let shaper = shaper_config();
    let ctrl: Shared = Arc::new(Mutex::new(Controller::new(
        chassi,
        Shaper::new(shaper),
        seed,
    )));
    {
        let mut controller = lock_recover(&ctrl);
        // Peak hour gives configured full intensity before the first NTP sync.
        controller.set_hour((shaper.peak_start + shaper.peak_end) / 2.0);
        for &name in cfg::ACTIVE_PATTERNS {
            controller.enable(name);
        }
        info!(
            "engine up: {} logical leds, {} output leds, {} active patterns; free heap = {} bytes",
            controller.leds().len(),
            cfg::LED_COUNT,
            cfg::ACTIVE_PATTERNS.len(),
            unsafe { esp_idf_svc::sys::esp_get_free_heap_size() }
        );
    }

    // --- Ethernet. Construction failures require a reboot to reacquire pins,
    // but patterns remain active for a minute before that recovery action. ---
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
    let now_ms = monotonic_ms();
    let (mut network, hardware_restart_at) =
        match net::NetworkManager::new(eth_pins, sysloop, health.clone(), now_ms) {
            Ok(manager) => (Some(manager), None),
            Err(error) => {
                warn!("ethernet construction failed: {error:?}; patterns remain active");
                health.update(|state| {
                    state.ethernet_driver = "failed";
                    state.ethernet_link = "down";
                    state.dhcp = "unavailable";
                    state.last_error = Some(format!("Ethernet construction failed: {error:?}"));
                });
                (None, Some(now_ms.saturating_add(HARDWARE_RESTART_MS)))
            }
        };

    let mut timesync = time::TimeSync::new(health.clone());

    // --- Optional runtime services ---
    let ddp_state = ddp::DdpState::new(cfg::LED_COUNT);
    let mut ddp_service = ddp::DdpService::new(ddp_state.clone(), health.clone());
    let header_pins: Vec<(u8, esp_idf_hal::gpio::AnyInputPin<'static>)> = vec![
        (17, pins.gpio17.degrade_input()),
        (32, pins.gpio32.degrade_input()),
        (33, pins.gpio33.degrade_input()),
        (34, pins.gpio34.degrade_input()),
        (35, pins.gpio35.degrade_input()),
        (36, pins.gpio36.degrade_input()),
    ];
    let mut buttons = buttons::Buttons::new(header_pins, ctrl.clone(), health.clone());
    let mut http = http::HttpManager::new(ctrl.clone(), ddp_state.clone(), health.clone());

    // --- Render + supervision loop ---
    let start_us = unsafe { esp_timer_get_time() };
    let frame_ms = (1000 / FPS).max(1);
    let mut mapped_leds = Vec::with_capacity(cfg::LED_COUNT);
    let mut next_supervision_ms = 0;
    let mut output_failures = FailureWindow::default();
    let mut last_output_log_ms = 0;

    loop {
        let now_ms = monotonic_ms();
        let now_us = unsafe { esp_timer_get_time() };
        let elapsed_us = now_us.saturating_sub(start_us) as u64;

        if now_ms >= next_supervision_ms {
            let network_status = network
                .as_mut()
                .map_or_else(net::NetworkStatus::default, |network| network.poll(now_ms));
            if let Some(hour) = timesync.poll(now_ms, network_status) {
                lock_recover(&ctrl).set_hour(hour);
            }
            ddp_service.poll(now_ms, network_status.ip_up);
            http.poll(now_ms, network_status.ip_up);
            next_supervision_ms = now_ms.saturating_add(SUPERVISOR_PERIOD_MS);

            if hardware_restart_at.is_some_and(|deadline| now_ms >= deadline) {
                controlled_restart("Ethernet hardware could not be constructed");
            }
        }

        buttons.poll(now_ms);
        {
            let mut controller = lock_recover(&ctrl);
            if ddp_state.active() {
                ddp_state.apply(&strand_map, controller.leds_mut());
            } else {
                controller.tick_micros(elapsed_us);
            }
            strand_map.copy_pixels(controller.leds(), &mut mapped_leds);
        }

        match leds.write(&mapped_leds) {
            Ok(()) => {
                if output_failures.consecutive() > 0 {
                    info!(
                        "LED output recovered after {} failed frames",
                        output_failures.consecutive()
                    );
                }
                output_failures.record_success();
                health.update(|state| state.consecutive_output_errors = 0);
            }
            Err(output_error) => {
                output_failures.record_failure(now_ms);
                let consecutive = output_failures.consecutive();
                health.update(|state| {
                    state.consecutive_output_errors = consecutive;
                    state.total_output_errors = state.total_output_errors.wrapping_add(1);
                    state.last_error = Some(format!("LED output failed: {output_error:?}"));
                });
                if consecutive == 1 || now_ms.saturating_sub(last_output_log_ms) >= 60_000 {
                    warn!("LED output failed ({output_error:?}); continuing render loop");
                    last_output_log_ms = now_ms;
                }
                if output_failures.expired(now_ms, OUTPUT_RESTART_MS) {
                    controlled_restart("LED output failed continuously for 10 seconds");
                }
            }
        }

        health.update(|state| state.frames = state.frames.wrapping_add(1));
        feed_watchdog();
        FreeRtos::delay_ms(frame_ms);
    }
}

fn configure_watchdog(health: &Health) {
    let config = esp_task_wdt_config_t {
        timeout_ms: 10_000,
        idle_core_mask: 0b11,
        trigger_panic: true,
    };
    let mut result = unsafe { esp_task_wdt_reconfigure(&config) };
    if result == ESP_ERR_INVALID_STATE {
        result = unsafe { esp_task_wdt_init(&config) };
    }
    if result != ESP_OK {
        warn!("watchdog configuration failed: {result}");
        health.record_error(format!("watchdog configuration failed: {result}"));
        return;
    }
    let result = unsafe { esp_task_wdt_add(core::ptr::null_mut()) };
    if result != ESP_OK {
        warn!("watchdog task registration failed: {result}");
        health.record_error(format!("watchdog registration failed: {result}"));
    } else {
        info!("watchdog: render task armed for 10 seconds");
    }
}

fn feed_watchdog() {
    let _ = unsafe { esp_task_wdt_reset() };
}

fn controlled_restart(reason: &str) -> ! {
    error!("{reason}; restarting");
    FreeRtos::delay_ms(100);
    unsafe { esp_restart() }
}

fn monotonic_ms() -> u64 {
    unsafe { esp_timer_get_time().max(0) as u64 / 1_000 }
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

fn map_ws281x_outputs(
    by_gpio: &mut [(u8, Option<AnyOutputPin<'static>>); 8],
) -> Vec<(AnyOutputPin<'static>, core::ops::Range<usize>)> {
    let mut outputs = Vec::new();
    for output in cfg::LED_OUTPUTS {
        let LedOutput::Ws281x { data, start, end } = *output else {
            warn!("ledinfo: ignoring APA102 mapping for WS281x driver");
            continue;
        };
        if let Some(pin) = take_led_pin(by_gpio, data) {
            outputs.push((pin, start..end));
        }
    }
    outputs
}

fn take_led_pin(
    by_gpio: &mut [(u8, Option<AnyOutputPin<'static>>); 8],
    gpio: u8,
) -> Option<AnyOutputPin<'static>> {
    if cfg!(feature = "uart-logs") && gpio == 1 {
        info!("ledinfo: uart-logs enabled; reserving GPIO1 for UART0 TX");
        return None;
    }
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
