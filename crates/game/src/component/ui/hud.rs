use bevy::{
    color::Color,
    ecs::{
        component::{Component, Immutable, StorageType},
        entity::Entity,
        template::FromTemplate,
    },
    prelude::Deref,
};
use lightyear::prelude::input::bei::InputAction;
use raid_race_lib::input;

#[derive(Component, FromTemplate, Deref)]
#[component(immutable)]
pub struct HealthBar(pub Entity);

impl HealthBar {
    pub const RED: Color = Color::srgb_u8(180, 0, 0);
    pub const PURPLE: Color = Color::srgb_u8(160, 0, 255);
}

#[derive(Component, FromTemplate, Deref)]
#[component(immutable)]
pub struct ManaBar(pub Entity);

#[derive(FromTemplate, Deref)]
pub struct Ability<const N: u8>(pub Entity);

impl<const N: u8> Component for Ability<N>
where
    input::Ability<N>: InputAction,
{
    type Mutability = Immutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;
}
