extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use embassy_time::{Duration, Instant, Timer};
use embedded_3dgfx::{
    Z_MAX_VALUE,
    command_buffer::CommandBuffer,
    config::apply_default_caps,
    engine::K3dengine,
    mesh::{Geometry, K3dMesh, RenderMode},
    renderer::FrameCtx,
};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb565, RgbColor},
    prelude::WebColors,
};

use nalgebra::Point3;

use crate::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH, FrameBuffer, render_buffer};

fn make_cube() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
    let vertices = vec![
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
    ];

    let faces = vec![
        [0, 1, 2],
        [0, 2, 3],
        [5, 4, 7],
        [5, 7, 6],
        [3, 2, 6],
        [3, 6, 7],
        [4, 5, 1],
        [4, 1, 0],
        [1, 5, 6],
        [1, 6, 2],
        [4, 0, 3],
        [4, 3, 7],
    ];

    (vertices, faces)
}

pub async fn _render(mut framebuffer: FrameBuffer) -> ! {
    let mut zbuffer = vec![Z_MAX_VALUE; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize];
    let mut commands = CommandBuffer::<100>::new();

    // Create 3D engine
    let mut engine = K3dengine::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
    apply_default_caps(&mut engine);
    engine.camera.set_position(Point3::new(0.0, 2.0, 6.0));
    engine.camera.set_target(Point3::new(0.0, 0.0, 0.0));

    // Create cube
    let (vertices, faces) = make_cube();
    let geometry = Geometry {
        vertices: &vertices,
        faces: &faces,
        colors: &[],
        lines: &[],
        normals: &[],
        vertex_normals: &[],
        uvs: &[],
        texture_id: None,
    };

    let mut cube = K3dMesh::new(geometry);
    cube.set_render_mode(RenderMode::Lines);
    cube.set_color(Rgb565::CSS_CYAN);

    let start_time = Instant::now();

    loop {
        // Calculate rotation based on time
        let elapsed = start_time.elapsed().as_secs() as f32;

        // Update cube rotation
        cube.set_attitude(elapsed * 0.5, elapsed, elapsed * 0.3);

        // Clear display
        framebuffer.clear(Rgb565::BLACK).unwrap();
        zbuffer.fill(Z_MAX_VALUE);

        engine
            .record(core::iter::once(&cube), &mut commands, None)
            .unwrap();
        let mut frame = FrameCtx {
            zbuffer: &mut zbuffer,
            width: DISPLAY_WIDTH as usize,
            height: DISPLAY_HEIGHT as usize,
        };
        engine
            .execute(&mut framebuffer, &mut frame, &commands, None)
            .unwrap();

        framebuffer = render_buffer(framebuffer).await;
        Timer::after(Duration::from_millis(16)).await;
    }
}
