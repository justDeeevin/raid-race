use crate::entity;
use bevy::{
    asset::Assets,
    ecs::system::{Commands, ResMut},
    mesh::Mesh,
    pbr::StandardMaterial,
};

pub fn player(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(entity::player(meshes, materials));
}
