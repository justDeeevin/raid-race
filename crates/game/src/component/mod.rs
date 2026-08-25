pub mod ui;
pub mod weapon;

use bevy::ecs::component::Component;
use raid_race_lib::component::alive::Id;

#[derive(Component)]
pub struct OrbitCamera(pub Id);

#[derive(Component)]
pub struct AimCamera;
