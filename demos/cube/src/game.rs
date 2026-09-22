use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::system::{Commands, Local, Query, ResMut},
};
use embedded_3dgfx::pipeline::vertex::{
    mesh::{K3dMesh, RenderMode},
    shapes::UNIT_CUBE,
};
use embedded_graphics::pixelcolor::{Rgb565, WebColors};
use nalgebra::{ComplexField, Point3, Vector3};

use crate::{mesh::Mesh3d, render::Engine};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Update, update);
    }
}

fn setup(mut commands: Commands, mut engine: ResMut<Engine>) {
    engine.camera.set_position(Point3::new(0.0, 2.0, 6.0));
    engine.camera.set_target(Point3::new(0.0, 0.0, 0.0));

    let mut cube = K3dMesh::new(UNIT_CUBE.geometry());
    cube.set_color(Rgb565::CSS_CYAN);
    cube.set_scale(3.0);

    let light_angle_h = 0.0f32; // horizontal
    let light_angle_v = 0.0f32; // vertical
    let light_dir =
        Vector3::new(light_angle_h.cos(), light_angle_v, light_angle_h.sin()).normalize();
    cube.set_render_mode(RenderMode::SolidLightDir(light_dir));
    // cube.set_render_mode(RenderMode::BlinnPhong {
    //     light_dir,
    //     specular_intensity: 0.8,
    //     shininess: 32.0,
    // });
    commands.spawn(Mesh3d(cube));
}

fn update(mut rotation: Local<f32>, cubes: Query<&mut Mesh3d>) {
    *rotation += 0.1;
    for mut cube in cubes {
        cube.set_attitude(*rotation * 0.5, *rotation, *rotation * 0.3);
    }
}
