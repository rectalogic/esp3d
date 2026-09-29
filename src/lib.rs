#![no_std]
pub mod app;
mod swapchain;
mod touch;

pub use ft6x36::TouchPoint;
pub use swapchain::{DISPLAY_HEIGHT, DISPLAY_WIDTH, SwapChain, FrameBuffer,};
pub use touch::{TouchReceiver,TouchEvent };
