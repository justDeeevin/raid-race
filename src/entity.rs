use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LockedAxes, RigidBody},
};
use bevy::{
    asset::Assets,
    color::Color,
    ecs::{bundle::Bundle, system::ResMut},
    math::primitives::Cuboid,
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    transform::components::Transform,
};

use crate::component::alive::{Agility, Cdr, Dps, Health, player::PlayerMovable};

pub fn player(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> impl Bundle {
    (
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        LockedAxes::ROTATION_LOCKED,
        PlayerMovable,
        Health(100),
        Dps(20),
        Agility(10),
        Cdr(0),
        Transform::from_xyz(0.0, 2.0, 0.0),
    )
}
