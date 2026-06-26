//! WS281x output over the modern (ESP-IDF v5) RMT encoder API.
//!
//! One `TxChannelDriver` (RMT channel, allocated from the v5 pool) per board
//! output. Each channel owns a pixel index range of the flat strand; on `write`
//! we encode that slice to WS281x bytes (`slisko_core::output`) and hand them to
//! a `BytesEncoder` configured with the chip's bit0/bit1 timing symbols.
//!
//! APA102 (clocked, SPI) will be a sibling module; this one is the bong69's
//! native clockless path.

use core::ops::Range;
use core::time::Duration;

use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::rmt::config::{TransmitConfig, TxChannelConfig};
use esp_idf_hal::rmt::encoder::{BytesEncoder, BytesEncoderConfig};
use esp_idf_hal::rmt::{PinState, Pulse, Symbol, TxChannelDriver};
use esp_idf_hal::sys::EspError;
use esp_idf_hal::units::{FromValueType, Hertz};

use slisko_core::output::{ColorOrder, LedType, encode_ws281x};
use slisko_core::pixel::Pixel;

/// RMT tick resolution: 10 MHz → 100 ns/tick (the espressif led_strip default;
/// well within WS281x ±150 ns tolerance).
const RESOLUTION_MHZ: u32 = 10;

/// WS281x bit timing in nanoseconds: high/low durations for a `0` and a `1` bit.
#[derive(Clone, Copy)]
struct BitTiming {
    t0h: u64,
    t0l: u64,
    t1h: u64,
    t1l: u64,
}

/// Bit timing per chip. One 800 kbps WS281x profile for now; per-type tuning
/// (e.g. WS2811 low-speed, WS2815 exact widths) is a TODO once we can scope it
/// on hardware.
fn timing_for(_t: LedType) -> BitTiming {
    BitTiming {
        t0h: 350,
        t0l: 800,
        t1h: 700,
        t1l: 600,
    }
}

fn encoder_config(t: LedType, res: Hertz) -> Result<BytesEncoderConfig, EspError> {
    let bt = timing_for(t);
    let bit0 = Symbol::new(
        Pulse::new_with_duration(res, PinState::High, Duration::from_nanos(bt.t0h))?,
        Pulse::new_with_duration(res, PinState::Low, Duration::from_nanos(bt.t0l))?,
    );
    let bit1 = Symbol::new(
        Pulse::new_with_duration(res, PinState::High, Duration::from_nanos(bt.t1h))?,
        Pulse::new_with_duration(res, PinState::Low, Duration::from_nanos(bt.t1l))?,
    );
    Ok(BytesEncoderConfig {
        bit0,
        bit1,
        msb_first: true, // WS281x is MSB-first per byte.
        ..Default::default()
    })
}

struct Strip<'d> {
    tx: TxChannelDriver<'d>,
    range: Range<usize>,
    order: ColorOrder,
    enc_cfg: BytesEncoderConfig,
}

/// All WS281x outputs on the board (up to 8 RMT channels).
pub struct Ws281xOutput<'d> {
    strips: Vec<Strip<'d>>,
    scratch: Vec<u8>,
}

impl<'d> Ws281xOutput<'d> {
    /// Build one RMT channel per `(pin, pixel range)`. `ledtype` must be a
    /// clockless chip (use the APA102/SPI path otherwise).
    pub fn new(
        outputs: Vec<(AnyOutputPin<'d>, Range<usize>)>,
        ledtype: LedType,
    ) -> Result<Self, EspError> {
        let res: Hertz = RESOLUTION_MHZ.MHz().into();
        let ch_cfg = TxChannelConfig {
            resolution: res,
            ..Default::default()
        };
        let enc_cfg = encoder_config(ledtype, res)?;
        let order = ledtype.default_color_order();

        let mut strips = Vec::with_capacity(outputs.len());
        for (pin, range) in outputs {
            let tx = TxChannelDriver::new(pin, &ch_cfg)?;
            strips.push(Strip {
                tx,
                range,
                order,
                enc_cfg: enc_cfg.clone(),
            });
        }
        Ok(Self {
            strips,
            scratch: Vec::new(),
        })
    }

    /// Encode and transmit the pixel buffer to every channel (blocking per
    /// channel). Ranges are clamped to the buffer length defensively.
    pub fn write(&mut self, leds: &[Pixel]) -> Result<(), EspError> {
        let tx_cfg = TransmitConfig::default();
        for strip in &mut self.strips {
            let end = strip.range.end.min(leds.len());
            let start = strip.range.start.min(end);
            encode_ws281x(&leds[start..end], strip.order, &mut self.scratch);
            if self.scratch.is_empty() {
                continue;
            }
            let encoder = BytesEncoder::with_config(&strip.enc_cfg)?;
            strip.tx.send_and_wait(encoder, &self.scratch, &tx_cfg)?;
        }
        Ok(())
    }
}
