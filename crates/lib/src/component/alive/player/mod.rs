pub mod character;
pub mod weapon;

use avian3d::math::Scalar;
use bevy::prelude::*;
use lightyear::core::id::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Deref)]
#[component(immutable)]
pub struct Player(pub PeerId);

#[derive(Component)]
pub struct Grounded;

#[derive(Component, Serialize, Deserialize, Deref, DerefMut, Clone, Copy, Debug, PartialEq)]
pub struct Pitch(pub Scalar);

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct AttackCooldown(pub Timer);
