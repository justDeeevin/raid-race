pub mod ui;
pub mod weapon;

use bevy::prelude::*;

#[derive(Component, Deref)]
pub struct OrbitCamera(pub Entity);

#[derive(Component)]
pub struct AimCamera;
