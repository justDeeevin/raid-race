use bevy::{ecs::component::Component, transform::components::Transform};
use naia_bevy_shared::{Property, Replicate, Serde};

#[derive(Component, Replicate)]
pub struct SimSync {
    pub translation: Property<Vec3>,
    pub velocity: Property<DVec3>,
}

impl SimSync {
    pub fn new(translation: bevy::math::Vec3, velocity: bevy::math::DVec3) -> Self {
        Self::new_complete(translation.into(), velocity.into())
    }
}

impl Default for SimSync {
    fn default() -> Self {
        Self::new_complete(Vec3::default(), DVec3::default())
    }
}

impl From<Transform> for SimSync {
    fn from(value: Transform) -> Self {
        Self::new_complete(value.translation.into(), DVec3::default())
    }
}

#[derive(Serde, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<bevy::math::Vec3> for Vec3 {
    fn from(value: bevy::math::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vec3> for bevy::math::Vec3 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Serde, Clone, Copy, PartialEq, Default)]
pub struct DVec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<bevy::math::DVec3> for DVec3 {
    fn from(value: bevy::math::DVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<DVec3> for bevy::math::DVec3 {
    fn from(value: DVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
