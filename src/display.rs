#![deny(clippy::large_stack_frames)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::convert::AsRef;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Delay;
use embedded_graphics::{
    pixelcolor::{Rgb565, RgbColor, raw::RawU16},
    prelude::IntoStorage,
};
use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};
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

type Display<'a> =
    AsyncDisplay<AsyncSpiInterface<ExclusiveDevice<SpiDma<'a, Async>, NoPin, NoDelay>, Output<'a>>>;

static RENDER_CHANNEL: Channel<CriticalSectionRawMutex, FrameBuffer, 1> = Channel::new();
static RECYCLE_CHANNEL: Channel<CriticalSectionRawMutex, FrameBuffer, 1> = Channel::new();

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

pub async fn initialize(spawner: &Spawner, peripherals: Peripherals) -> FrameBuffer {
    spawner.spawn(render_task(peripherals).expect("spawn render_task"));
    RECYCLE_CHANNEL.receive().await
}

#[embassy_executor::task]
async fn render_task(peripherals: Peripherals) {
    let mut display = new_display(peripherals).await;
    RECYCLE_CHANNEL.send(new_framebuffer()).await;
    loop {
        let framebuffer = RENDER_CHANNEL.receive().await;
        display
            .write_pixels_raw(framebuffer.data.as_ref())
            .await
            .expect("write_pixels_raw");
        RECYCLE_CHANNEL.send(framebuffer).await;
    }
}

pub async fn render_buffer(framebuffer: FrameBuffer) -> FrameBuffer {
    RENDER_CHANNEL.send(framebuffer).await;
    RECYCLE_CHANNEL.receive().await
}

async fn new_display<'a>(peripherals: Peripherals) -> Display<'a> {
    let dma_rx_buf = dma_rx_buffer!(4).unwrap();
    let dma_tx_buf = dma_tx_buffer!(32000).unwrap();

    let spi_bus = Spi::new(
        peripherals.spi2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
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

const FRAMEBUFFER_SIZE: usize = FRAMEBUFFER_WIDTH as usize * FRAMEBUFFER_HEIGHT as usize;

pub struct OwnedBuffer(Box<[Rgb565]>);

impl FrameBufferBackend for OwnedBuffer {
    type Color = Rgb565;

    fn set(&mut self, index: usize, color: Rgb565) {
        self.0[index] = Rgb565::from(RawU16::from(color.into_storage().swap_bytes()));
    }

    fn get(&self, index: usize) -> Rgb565 {
        Rgb565::from(RawU16::from(self.0[index].into_storage().swap_bytes()))
    }

    fn nr_elements(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<[u8]> for OwnedBuffer {
    fn as_ref(&self) -> &[u8] {
        let pixels = self.0.as_ref();
        let ptr = pixels.as_ptr() as *const u8;
        let len = size_of_val(pixels);
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }
}

pub type FrameBuffer = FrameBuf<Rgb565, OwnedBuffer>;

fn new_framebuffer() -> FrameBuffer {
    let pixels = vec![Rgb565::BLACK; FRAMEBUFFER_SIZE].into_boxed_slice();

    FrameBuffer::new(
        OwnedBuffer(pixels),
        FRAMEBUFFER_HEIGHT as usize,
        FRAMEBUFFER_WIDTH as usize,
    )
}

struct NoPin;

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
