pub mod player;
pub mod status;

use bevy::{
    ecs::component::Component,
    prelude::{Deref, DerefMut},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Deref)]
#[component(immutable)]
/// Unique identifier for living entities
pub struct Id(pub u64);

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Health(pub Meter);

impl Health {
    pub fn new(cap: u16) -> Self {
        Self(Meter::new(cap))
    }
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Mana(pub Meter);

impl Mana {
    pub fn new(cap: u16) -> Self {
        Self(Meter::new(cap))
    }
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Dps(pub u16);

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
/// Increases movement speed and attack rate[^1].
///
/// [^1]: DPS remains respected; thus, an increased agility will decrease the damage per hit.
pub struct Agility(pub u8);

impl Agility {
    pub const MOVE_SPEED_ADJUST: f32 = 0.1;
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
/// Cooldown reduction
pub struct Cdr(pub u16);

impl Cdr {
    /// Returns reduction factor for cooldowns.
    ///
    /// The max reduction is 30%. The value increases rationally (k = 15) with an asymptote at the max.
    pub fn scaler(&self) -> f32 {
        const MAX_REDUCTION: f32 = 0.3;
        const HALFWAY_POINT: f32 = 15.0;

        let x = **self as f32;

        // rational function
        let f: f32 = x / (x + HALFWAY_POINT);

        1.0 - (MAX_REDUCTION * f)
    }
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Defense(pub i16);

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
/// A percent between 0 and 100
pub struct Luck(pub u8);

#[derive(Serialize, Deserialize)]
pub struct Meter {
    pub cap: u16,
    pub current: u16,
}

impl Meter {
    pub fn new(cap: u16) -> Self {
        Self { cap, current: cap }
    }
}
