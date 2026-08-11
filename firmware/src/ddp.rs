//! DDP (Distributed Display Protocol) sink + the `PixelSource` toggle.
//!
//! Lets an external controller (xLights, Falcon, or slisko-on-RPI via `--ddp`)
//! override the internal patterns: a UDP listener on port 4048 fills a shared
//! RGB buffer, and when DDP mode is active the render loop paints that buffer
//! instead of ticking patterns. Packet parsing is delegated to `ddp-rs`'
//! allocation-free `PacketRef`, which handles both the 10-byte and 14-byte
//! (timecode) header variants and exposes the byte offset / length / payload.

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use ddp_rs::packet::PacketRef;
use esp_idf_svc::sys::esp_timer_get_time;
use log::{info, warn};

use engine::output::StrandMap;
use engine::pixel::Pixel;

use crate::health::{Health, ServiceState, lock_recover};
use crate::recovery::ExponentialBackoff;

pub const DDP_PORT: u16 = 4048;
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
        now.saturating_sub(*lock_recover(&self.last_us)) < STALE_US
    }

    /// Paint the latest DDP RGB buffer into the strand.
    pub fn apply(&self, map: &StrandMap, leds: &mut [Pixel]) {
        let buf = lock_recover(&self.rgb);
        let _ = map.apply_srgb8(&buf, leds);
    }

    fn ingest(&self, offset: usize, data: &[u8]) {
        let mut buf = lock_recover(&self.rgb);
        let end = offset.saturating_add(data.len()).min(buf.len());
        if offset < end {
            buf[offset..end].copy_from_slice(&data[..end - offset]);
        }
        *lock_recover(&self.last_us) = unsafe { esp_timer_get_time() };
    }
}

pub struct DdpService {
    state: Arc<DdpState>,
    health: Health,
    worker: Option<JoinHandle<()>>,
    retry: ExponentialBackoff,
}

impl DdpService {
    pub fn new(state: Arc<DdpState>, health: Health) -> Self {
        Self {
            state,
            health,
            worker: None,
            retry: ExponentialBackoff::new(1_000, 60_000),
        }
    }

    pub fn poll(&mut self, now_ms: u64, network_ready: bool) {
        if self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            let Some(worker) = self.worker.take() else {
                return;
            };
            let detail = match worker.join() {
                Ok(()) => "DDP worker exited".to_string(),
                Err(_) => "DDP worker panicked".to_string(),
            };
            warn!("{detail}; scheduling restart");
            self.health.update(|health| {
                health.ddp = ServiceState::Stopped;
                health.last_error = Some(detail);
            });
            self.retry.fail(now_ms);
        }

        // Binding a socket before ESP-IDF has constructed a usable esp-netif
        // instance aborts inside lwIP (`Invalid mbox`) instead of returning an
        // ordinary I/O error. Existing workers may remain alive across a link
        // outage because the TCP/IP stack still exists, but never create the
        // first worker until DHCP has brought that stack fully online.
        if !network_ready {
            if self.worker.is_none() {
                self.health
                    .update(|health| health.ddp = ServiceState::Stopped);
            }
            return;
        }

        if self.worker.is_none() && self.retry.ready(now_ms) {
            let state = self.state.clone();
            let health = self.health.clone();
            match std::thread::Builder::new()
                .name("ddp".into())
                // The receive buffer alone is 1500 bytes; leave enough room
                // for Rust/ESP-IDF socket and logging frames during failures.
                .stack_size(8192)
                .spawn(move || run(state, health))
            {
                Ok(worker) => {
                    self.worker = Some(worker);
                    self.retry.reset();
                    self.health
                        .update(|health| health.ddp = ServiceState::Running);
                }
                Err(error) => {
                    let delay = self.retry.fail(now_ms);
                    warn!("ddp: worker spawn failed ({error}); retrying in {delay} ms");
                    self.health.update(|health| {
                        health.ddp = ServiceState::Retrying;
                        health.last_error = Some(format!("DDP spawn failed: {error}"));
                    });
                }
            }
        }
    }
}

fn run(state: Arc<DdpState>, health: Health) {
    let mut packet = [0u8; 1500];
    loop {
        let sock = match UdpSocket::bind(("0.0.0.0", DDP_PORT)) {
            Ok(s) => s,
            Err(e) => {
                warn!("ddp: bind :{DDP_PORT} failed ({e}); retrying");
                health.update(|state| {
                    state.ddp = ServiceState::Retrying;
                    state.last_error = Some(format!("DDP bind failed: {e}"));
                });
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
        };
        if let Err(error) = sock.set_read_timeout(Some(Duration::from_secs(1))) {
            warn!("ddp: failed to set receive timeout ({error})");
        }
        info!("ddp: listening on udp/{DDP_PORT}");
        health.update(|state| state.ddp = ServiceState::Running);
        loop {
            match sock.recv(&mut packet) {
                Ok(n) => {
                    // `PacketRef` parses the header (10- or 14-byte) and borrows
                    // the payload; the offset is a byte offset into the strand.
                    if let Some(p) = PacketRef::from_bytes(&packet[..n]) {
                        let avail = (p.header.length as usize).min(p.data.len());
                        state.ingest(p.header.offset as usize, &p.data[..avail]);
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    health.update(|state| state.ddp = ServiceState::Running);
                }
                Err(e) => {
                    warn!("ddp: recv error ({e}); rebinding");
                    health.update(|state| {
                        state.ddp = ServiceState::Retrying;
                        state.last_error = Some(format!("DDP receive failed: {e}"));
                    });
                    break;
                }
            }
        }
    }
}
