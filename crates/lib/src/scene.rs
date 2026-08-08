use avian3d::{
    collision::collider::Collider, dynamics::rigid_body::RigidBody, physics_transform::Position,
};
use bevy::{
    asset::asset_value,
    color::Color,
    light::PointLight,
    math::primitives::Cuboid,
    mesh::Mesh3d,
    pbr::{MeshMaterial3d, StandardMaterial},
    scene::{SceneList, bsn_list, template_value},
    transform::components::Transform,
};

pub fn test() -> impl SceneList {
    bsn_list![
        (
            #Floor
            template_value(RigidBody::Static)
            Collider::cuboid(100.0, 1.0, 100.0)
            Mesh3d(asset_value(Cuboid::new(100.0, 1.0, 100.0)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
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
    ]
}
