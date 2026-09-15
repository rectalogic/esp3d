#![deny(clippy::large_stack_frames)]

use embassy_time::Delay;
use embedded_hal::digital::{ErrorType, OutputPin};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::{
    Async,
    dma::AhbGdmaChannel,
    dma_rx_buffer, dma_tx_buffer,
    gpio::{AnyPin, Level, Output, OutputConfig},
    peripherals::SPI2,
    spi::{
        Mode as SpiMode,
        master::{Config as SpiConfig, Spi, SpiDma},
    },
    time::Rate,
};

use sky_ili9341::{
    AsyncBuilder, AsyncDisplay, AsyncSpiInterface, ColorInversion, ColorOrder, Orientation,
    options::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH},
};

// Rotated landscape
pub const DISPLAY_WIDTH: u16 = FRAMEBUFFER_HEIGHT;
pub const DISPLAY_HEIGHT: u16 = FRAMEBUFFER_WIDTH;

pub type Display<'a> =
    AsyncDisplay<AsyncSpiInterface<ExclusiveDevice<SpiDma<'a, Async>, NoPin, NoDelay>, Output<'a>>>;

pub struct Peripherals {
    pub spi2: SPI2<'static>,
    pub dma: AhbGdmaChannel<'static>,
    pub dc: AnyPin<'static>,
    // miso GPIO11 not needed
    pub mosi: AnyPin<'static>,
    pub sclk: AnyPin<'static>,
    pub cs: AnyPin<'static>,
    pub bl: AnyPin<'static>,
}

#[expect(clippy::large_stack_frames)]
pub async fn new_display<'a>(peripherals: Peripherals) -> Display<'a> {
    let dma_rx_buf = dma_rx_buffer!(4).unwrap();
    let dma_tx_buf = dma_tx_buffer!(32000).unwrap();

    let spi_bus = Spi::new(
        peripherals.spi2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(60))
            .with_mode(SpiMode::_0),
    )
    .expect("display SPI")
    .with_sck(peripherals.sclk)
    .with_mosi(peripherals.mosi)
    .with_cs(peripherals.cs)
    .with_dma(peripherals.dma)
    .with_buffers(dma_rx_buf, dma_tx_buf)
    .into_async();

    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, NoPin).expect("infallible");

    let dc = Output::new(peripherals.dc, Level::Low, OutputConfig::default());

    let di = AsyncSpiInterface::new(spi_device, dc);
    let mut delay = Delay;
    let mut reset_pin = NoPin;
    let mut display = AsyncBuilder::new(di)
        .invert_colors(ColorInversion::Inverted)
        .color_order(ColorOrder::Bgr)
        .orientation(Orientation::Landscape)
        .init(&mut reset_pin, &mut delay)
        .await
        .expect("display builder init");

    let _backlight = Output::new(peripherals.bl, Level::High, OutputConfig::default());
    display.clear_screen(0x0000).await.expect("display clear");

    display
}

pub struct NoPin;

impl OutputPin for NoPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl ErrorType for NoPin {
    type Error = core::convert::Infallible;
}
