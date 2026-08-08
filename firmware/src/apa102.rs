//! APA102 output over SPI (the clocked sibling of the WS281x RMT path).
//!
//! APA102 consumes two output-capable GPIOs per chain. Frame bytes come from
//! `engine::output::Apa102Encoder`; this module clocks each configured pixel
//! range out on its independent SPI2/SPI3 chain without chip-select.

use core::ops::Range;

use esp_idf_hal::gpio::{AnyInputPin, AnyOutputPin, OutputPin};
use esp_idf_hal::spi::config::{Config, DriverConfig, MODE_3};
use esp_idf_hal::spi::{SpiAnyPins, SpiDeviceDriver, SpiDriver};
use esp_idf_hal::sys::EspError;
use esp_idf_hal::units::FromValueType;

use engine::output::{Apa102Encoder, Apa102Options};
use engine::pixel::Pixel;

pub struct Apa102Output<'d> {
    dev: SpiDeviceDriver<'d, SpiDriver<'d>>,
    range: Range<usize>,
    encoder: Apa102Encoder,
    scratch: Vec<u8>,
}

impl<'d> Apa102Output<'d> {
    /// Create an APA102 output on the given SPI peripheral + clock/data pins.
    pub fn new<SPI: SpiAnyPins + 'd>(
        spi: SPI,
        sclk: impl OutputPin + 'd,
        mosi: impl OutputPin + 'd,
        range: Range<usize>,
        options: Apa102Options,
    ) -> Result<Self, EspError> {
        let driver = SpiDriver::new(spi, sclk, mosi, None::<AnyInputPin>, &DriverConfig::new())?;
        let config = Config::new()
            .baudrate(8.MHz().into())
            .data_mode(MODE_3)
            .write_only(true);
        let dev = SpiDeviceDriver::new(driver, None::<AnyOutputPin>, &config)?;
        Ok(Self {
            dev,
            range,
            encoder: Apa102Encoder::new(options),
            scratch: Vec::new(),
        })
    }

    /// Encode the strand as an APA102 frame and clock it out.
    pub fn write(&mut self, leds: &[Pixel]) -> Result<(), EspError> {
        let end = self.range.end.min(leds.len());
        let start = self.range.start.min(end);
        self.encoder.encode(&leds[start..end], &mut self.scratch);
        self.dev.write(&self.scratch)
    }
}
