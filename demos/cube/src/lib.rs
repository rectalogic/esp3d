#![no_std]

mod game;
mod mesh;
mod render;

use bevy::{DefaultPlugins, app::App, platform::time::Instant as BevyInstant};
use embassy_time::{Duration, Instant};

pub async fn render(swapchain: esp3d::SwapChain, touch_receiver: esp3d::TouchReceiver) -> ! {
    unsafe {
        BevyInstant::set_elapsed(|| Duration::from_ticks(Instant::now().as_ticks()).into());
    }
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, render::RenderPlugin, game::GamePlugin));
    render::run(app, swapchain, touch_receiver, 30).await
}
