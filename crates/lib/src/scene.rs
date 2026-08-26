use crate::{
    component::alive::Health,
    system::player::{PLAYER_CAPSULE_LENGTH, PLAYER_HEIGHT, PLAYER_RADIUS},
};
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component, FromTemplate)]
pub struct Dummy;

pub fn test() -> impl SceneList {
    bsn_list![
        (
            #Floor
            template_value(RigidBody::Static)
            Collider::cuboid(100.0, 1.0, 100.0)
            Mesh3d(asset_value(Cuboid::new(100.0, 1.0, 100.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            Friction::new(3.0)
        ),
        (
            #FrontWall
            template_value(RigidBody::Static)
            Position::from_xyz(0.0, 0.5, -50.0)
            Collider::cuboid(100.0, 4.0, 1.0)
            Mesh3d(asset_value(Cuboid::new(100.0, 4.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        ),
        (
            #RightWall
            template_value(RigidBody::Static)
            Position::from_xyz(50.0, 0.5, 0.0)
            Collider::cuboid(1.0, 4.0, 100.0)
            Mesh3d(asset_value(Cuboid::new(1.0, 4.0, 100.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        ),
        (
            #BackWall
            template_value(RigidBody::Static)
            Position::from_xyz(0.0, 0.5, 50.0)
            Collider::cuboid(100.0, 4.0, 1.0)
            Mesh3d(asset_value(Cuboid::new(100.0, 4.0, 1.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        ),
        (
            #LeftWall
            template_value(RigidBody::Static)
            Collider::cuboid(1.0, 4.0, 100.0)
            Mesh3d(asset_value(Cuboid::new(1.0, 4.0, 100.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
            Position::from_xyz(-50.0, 0.5, 0.0)
        ),
        (
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(0.0, 8.0, 0.0)
        ),
        (
            Dummy
            Health::new(u16::MAX)
            Collider::capsule(PLAYER_RADIUS, PLAYER_CAPSULE_LENGTH)
            Mesh3d(asset_value(Capsule3d::new(PLAYER_RADIUS as f32, PLAYER_CAPSULE_LENGTH as f32)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::BLACK))
            Position::from_xyz(0.0, (PLAYER_HEIGHT / 2.0) + 0.5, 3.0)
        )
    ]
}
