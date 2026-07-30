use bevy::{
    color::Color,
    ecs::{component::Component, entity::Entity},
};

#[derive(Component)]
pub struct HealthBar(pub Entity);

impl HealthBar {
    pub const RED: Color = Color::srgb_u8(180, 0, 0);
    pub const PURPLE: Color = Color::srgb_u8(160, 0, 255);
}

#[derive(Component)]
pub struct ManaBar(pub Entity);
