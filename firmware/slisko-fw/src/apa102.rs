//! APA102 output over SPI (the clocked sibling of the WS281x RMT path).
//!
//! APA102 needs clock + data, so it is NOT usable on the bong69's data-only
//! outputs — it's here for boards that wire an APA102 strip to an SPI bus. The
//! frame bytes come from `slisko_core::output::encode_apa102`; this just clocks
//! them out. Selected when `[ledinfo].type == "APA102"`.

use esp_idf_hal::gpio::{AnyInputPin, AnyOutputPin, OutputPin};
use esp_idf_hal::spi::config::{Config, DriverConfig};
use esp_idf_hal::spi::{SpiAnyPins, SpiDeviceDriver, SpiDriver};
use esp_idf_hal::sys::EspError;
use esp_idf_hal::units::FromValueType;

use slisko_core::output::{APA102_MAX_BRIGHTNESS, encode_apa102};
use slisko_core::pixel::Pixel;

pub struct Apa102Output<'d> {
    dev: SpiDeviceDriver<'d, SpiDriver<'d>>,
    brightness: u8,
    scratch: Vec<u8>,
}

impl<'d> Apa102Output<'d> {
    /// Create an APA102 output on the given SPI peripheral + clock/data pins.
    /// `brightness` is the APA102 5-bit global value (0..=31).
    pub fn new<SPI: SpiAnyPins + 'd>(
        spi: SPI,
        sclk: impl OutputPin + 'd,
        mosi: impl OutputPin + 'd,
        brightness: u8,
    ) -> Result<Self, EspError> {
        let driver = SpiDriver::new(spi, sclk, mosi, None::<AnyInputPin>, &DriverConfig::new())?;
        let config = Config::new().baudrate(8.MHz().into());
        let dev = SpiDeviceDriver::new(driver, None::<AnyOutputPin>, &config)?;
        Ok(Self {
            dev,
            brightness: brightness.min(APA102_MAX_BRIGHTNESS),
            scratch: Vec::new(),
        })
    }

    /// Encode the strand as an APA102 frame and clock it out.
    pub fn write(&mut self, leds: &[Pixel]) -> Result<(), EspError> {
        encode_apa102(leds, self.brightness, &mut self.scratch);
        self.dev.write(&self.scratch)
    }
}
