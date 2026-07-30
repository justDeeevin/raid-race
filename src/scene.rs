use avian3d::{collision::collider::Collider, dynamics::rigid_body::RigidBody};
use bevy::{
    asset::asset_value,
    camera::visibility::Visibility,
    color::Color,
    ecs::hierarchy::Children,
    light::PointLight,
    math::{Quat, primitives::Circle},
    mesh::Mesh3d,
    pbr::{MeshMaterial3d, StandardMaterial},
    scene::{SceneList, bsn_list, template_value},
    transform::components::Transform,
};

pub fn main() -> impl SceneList {
    bsn_list![
        (
            template_value(RigidBody::Static)
            Collider::cylinder(4.0, 0.0)
            // shouldn't be necessary but this gets rid of a runtime warning
            Visibility::Visible
            Children [
                Mesh3d(asset_value(Circle::new(4.0)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
                Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            ]
        ),
        (
            PointLight {
                shadow_maps_enabled: true,
            }
            Transform::from_xyz(4.0, 8.0, 4.0)
        ),
    ]
}
