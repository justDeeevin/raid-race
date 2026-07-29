use bevy::ecs::{component::Component, entity::Entity};

#[derive(Component)]
pub struct HealthBar(pub Entity);
