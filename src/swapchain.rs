#![deny(clippy::large_stack_frames)]

mod display;
pub use display::{DISPLAY_HEIGHT, DISPLAY_WIDTH, Peripherals};

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::convert::AsRef;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embedded_graphics::{
    pixelcolor::{Rgb565, RgbColor, raw::RawU16},
    prelude::IntoStorage,
};
use embedded_graphics_framebuf::{FrameBuf, backends::FrameBufferBackend};

use sky_ili9341::options::{FRAMEBUFFER_HEIGHT, FRAMEBUFFER_WIDTH};

static FRONTBUFFER_CHANNEL: Channel<CriticalSectionRawMutex, FrameBuffer, 1> = Channel::new();
static BACKBUFFER_CHANNEL: Channel<CriticalSectionRawMutex, FrameBuffer, 1> = Channel::new();

pub struct SwapChain {
    back_buffer: Option<FrameBuffer>,
}

impl SwapChain {
    pub async fn new() -> Self {
        BACKBUFFER_CHANNEL.send(new_framebuffer()).await;
        Self {
            back_buffer: Some(new_framebuffer()),
        }
    }

    pub fn back_buffer(&mut self) -> &mut FrameBuffer {
        self.back_buffer.as_mut().unwrap()
    }

    pub async fn present(&mut self) {
        FRONTBUFFER_CHANNEL
            .send(self.back_buffer.take().unwrap())
            .await;
        self.back_buffer = Some(BACKBUFFER_CHANNEL.receive().await);
    }
}

#[embassy_executor::task]
pub async fn swapchain_task(peripherals: display::Peripherals) {
    let mut display = display::new_display(peripherals).await;
    loop {
        let framebuffer = FRONTBUFFER_CHANNEL.receive().await;
        display
            .write_pixels_raw(framebuffer.data.as_ref())
            .await
            .expect("write_pixels_raw");
        BACKBUFFER_CHANNEL.send(framebuffer).await;
    }
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
