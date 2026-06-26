//! DDP (Distributed Display Protocol) sink + the `PixelSource` toggle.
//!
//! Lets an external controller (xLights, Falcon, or slisko-on-RPI via `--ddp`)
//! override the internal patterns: a UDP listener on port 4048 fills a shared
//! RGB buffer, and when DDP mode is active the render loop paints that buffer
//! instead of ticking patterns. DDP packet = 10-byte header + RGB data, with a
//! 32-bit byte offset and 16-bit length (both big-endian).

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use esp_idf_svc::sys::esp_timer_get_time;
use log::{info, warn};

use slisko_core::pixel::Pixel;

pub const DDP_PORT: u16 = 4048;
const HEADER_LEN: usize = 10;
/// If no DDP frame arrives within this window, fall back to internal patterns.
const STALE_US: i64 = 2_000_000;

pub struct DdpState {
    /// Latest RGB bytes (3 per pixel), indexed by strand position.
    rgb: Mutex<Vec<u8>>,
    /// Operator/API request to use DDP when frames are arriving.
    enabled: AtomicBool,
    /// esp_timer micros of the last received frame. (ESP32 has no 64-bit
    /// atomics, so this is a small Mutex rather than an AtomicI64.)
    last_us: Mutex<i64>,
}

impl DdpState {
    pub fn new(num_leds: usize) -> Arc<Self> {
        Arc::new(DdpState {
            rgb: Mutex::new(vec![0u8; num_leds * 3]),
            enabled: AtomicBool::new(true),
            last_us: Mutex::new(i64::MIN),
        })
    }

    pub fn set_enabled(&self, on: bool) {
        self.enabled.store(on, Ordering::Relaxed);
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// True when DDP is enabled AND a frame arrived recently.
    pub fn active(&self) -> bool {
        if !self.enabled() {
            return false;
        }
        let now = unsafe { esp_timer_get_time() };
        now - *self.last_us.lock().unwrap() < STALE_US
    }

    /// Paint the latest DDP RGB buffer into the strand.
    pub fn apply(&self, leds: &mut [Pixel]) {
        let buf = self.rgb.lock().unwrap();
        for (i, px) in leds.iter_mut().enumerate() {
            let o = i * 3;
            if o + 2 < buf.len() {
                px.set_color(
                    buf[o] as f32 / 255.0,
                    buf[o + 1] as f32 / 255.0,
                    buf[o + 2] as f32 / 255.0,
                );
            }
        }
    }

    fn ingest(&self, offset: usize, data: &[u8]) {
        let mut buf = self.rgb.lock().unwrap();
        let end = (offset + data.len()).min(buf.len());
        if offset < end {
            buf[offset..end].copy_from_slice(&data[..end - offset]);
        }
        *self.last_us.lock().unwrap() = unsafe { esp_timer_get_time() };
    }
}

/// Spawn the UDP receive loop. Errors (e.g. before the netif is up) are logged;
/// the thread keeps trying to (re)bind.
pub fn spawn(state: Arc<DdpState>) {
    std::thread::Builder::new()
        .name("ddp".into())
        .stack_size(4096)
        .spawn(move || run(state))
        .expect("spawn ddp thread");
}

fn run(state: Arc<DdpState>) {
    let mut packet = [0u8; 1500];
    loop {
        let sock = match UdpSocket::bind(("0.0.0.0", DDP_PORT)) {
            Ok(s) => s,
            Err(e) => {
                warn!("ddp: bind :{DDP_PORT} failed ({e}); retrying");
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
        };
        info!("ddp: listening on udp/{DDP_PORT}");
        loop {
            match sock.recv(&mut packet) {
                Ok(n) if n > HEADER_LEN => {
                    // offset: bytes 4..8 (BE), length: bytes 8..10 (BE).
                    let offset =
                        u32::from_be_bytes([packet[4], packet[5], packet[6], packet[7]]) as usize;
                    let len = u16::from_be_bytes([packet[8], packet[9]]) as usize;
                    let avail = (n - HEADER_LEN).min(len);
                    state.ingest(offset, &packet[HEADER_LEN..HEADER_LEN + avail]);
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("ddp: recv error ({e}); rebinding");
                    break;
                }
            }
        }
    }
}
