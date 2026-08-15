use bevy::{
    ecs::component::Component,
    prelude::{Deref, DerefMut},
    reflect::Reflect,
};
use lightyear::core::id::PeerId;
use serde::{Deserialize, Serialize};

#[derive(
    Component, Serialize, Deserialize, Reflect, Clone, Debug, PartialEq, Eq, Deref, DerefMut,
)]
pub struct Player(pub PeerId);

#[derive(Component)]
pub struct Grounded;

#[derive(
    Component, Serialize, Deserialize, Reflect, Deref, DerefMut, Clone, Copy, Debug, PartialEq,
)]
pub struct Pitch(pub f64);
