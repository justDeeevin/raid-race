pub mod ui;
pub mod weapon;

use bevy::{
    ecs::{component::Component, entity::Entity},
    prelude::Deref,
};

#[derive(Component, Deref)]
pub struct OrbitCamera(pub Entity);

#[derive(Component)]
pub struct AimCamera;
