extern crate alloc;

use alloc::vec;
use bevy::{
    app::{App, Last, MainScheduleOrder, Plugin},
    ecs::{
        resource::Resource,
        schedule::{Schedule, ScheduleLabel},
        system::{Query, Res, ResMut},
    },
    prelude::{Deref, DerefMut},
};
use embassy_time::{Duration, Ticker};
use embedded_3dgfx::{
    Z_MAX_VALUE,
    config::apply_default_caps,
    engine::K3dengine,
    pipeline::{command_buffer::CommandBuffer, renderer::FrameCtx},
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
};
use esp3d::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

use crate::mesh::Mesh3d;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Engine::new())
            .insert_resource(EngineCommands(CommandBuffer::<100>::new()))
            .add_schedule(Schedule::new(Render))
            .add_systems(Render, render)
            .world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(Last, Render);
    }
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct Render;

#[derive(Resource, Deref, DerefMut)]
pub struct Engine(K3dengine);

impl Engine {
    fn new() -> Self {
        let mut engine = K3dengine::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
        apply_default_caps(&mut engine);
        Self(engine)
    }
}

#[derive(Resource, Deref, DerefMut)]
struct EngineCommands(CommandBuffer<100>);

fn render(engine: Res<Engine>, mut command_buffer: ResMut<EngineCommands>, meshes: Query<&Mesh3d>) {
    engine
        .record(meshes.into_iter().map(|m| &m.0), &mut command_buffer, None)
        .unwrap();
}

pub async fn run(mut app: App, mut swapchain: esp3d::SwapChain, fps: u8) -> ! {
    app.finish();
    app.cleanup();

    let duration = Duration::from_millis(1000 / fps as u64);
    let mut ticker = Ticker::every(duration);
    let mut zbuffer = vec![Z_MAX_VALUE; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize];
    loop {
        // Clear display
        swapchain.back_buffer().clear(Rgb565::BLACK).unwrap();
        zbuffer.fill(Z_MAX_VALUE);

        app.update();

        let mut frame = FrameCtx {
            zbuffer: &mut zbuffer,
            width: DISPLAY_WIDTH as usize,
            height: DISPLAY_HEIGHT as usize,
        };

        app.world()
            .get_resource::<Engine>()
            .unwrap()
            .execute(
                swapchain.back_buffer(),
                &mut frame,
                app.world().get_resource::<EngineCommands>().unwrap(),
                None,
            )
            .unwrap();
        swapchain.present().await;

        ticker.next().await;
    }
}
