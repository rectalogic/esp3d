use bevy::{
    ecs::component::Component,
    prelude::{Deref, DerefMut},
};
use embedded_3dgfx::pipeline::vertex::mesh::K3dMesh;

#[derive(Component, Deref, DerefMut)]
pub struct Mesh3d(pub K3dMesh<'static>);
