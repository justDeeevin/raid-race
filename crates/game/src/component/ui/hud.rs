use bevy::{
    color::Color,
    ecs::{component::Component, entity::Entity, template::FromTemplate},
    prelude::{Deref, DerefMut},
    reflect::Reflect,
};

#[derive(Component, FromTemplate, Reflect, Deref, DerefMut)]
pub struct HealthBar(pub Entity);

impl HealthBar {
    pub const RED: Color = Color::srgb_u8(180, 0, 0);
    pub const PURPLE: Color = Color::srgb_u8(160, 0, 255);
}

#[derive(Component, FromTemplate, Deref, DerefMut)]
pub struct ManaBar(pub Entity);
