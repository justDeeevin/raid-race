pub mod ui;

use bevy::{
    ecs::{component::Component, entity::Entity},
    math::Vec3,
};

#[derive(Component)]
pub struct OrbitCamera {
    pub target: Entity,
    pub offset: Vec3,
}

impl OrbitCamera {
    pub const ORBIT_DISTANCE: f32 = 5.0;
}
