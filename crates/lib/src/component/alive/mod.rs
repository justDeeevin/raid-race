pub mod player;
// TODO: replicate status effects
pub mod status;

use bevy::{
    ecs::component::Component,
    prelude::{Deref, DerefMut},
};
use naia_bevy_shared::{Property, Replicate, Serde};

#[derive(Component, Replicate, Deref, DerefMut)]
pub struct Health(pub Property<Meter>);

impl Health {
    pub fn new(cap: u16) -> Self {
        Self::new_complete(Meter::new(cap))
    }
}

#[derive(Component, Replicate, Deref, DerefMut)]
pub struct Mana(pub Property<Meter>);

impl Mana {
    pub fn new(cap: u16) -> Self {
        Self::new_complete(Meter::new(cap))
    }
}

#[derive(Component, Replicate, Deref, DerefMut)]
pub struct Dps(pub Property<u16>);

#[derive(Component, Replicate, Deref, DerefMut)]
pub struct Agility(pub Property<u8>);

impl Agility {
    pub const MOVE_SPEED_ADJUST: f64 = 0.1;
}

#[derive(Component, Replicate, Deref, DerefMut)]
pub struct Cdr(pub Property<u16>);

impl Cdr {
    pub fn scale(&self) -> f32 {
        const MAX_REDUCTION: f32 = 0.3;
        const HALFWAY_POINT: f32 = 15.0;

        // WE LOVE INDIRECTION
        let x = ***self as f32;

        // rational function
        let f: f32 = x / (x + HALFWAY_POINT);

        1.0 - (MAX_REDUCTION * f)
    }
}

#[derive(Component, Replicate, Deref, DerefMut)]
pub struct Defense(pub Property<i16>);

#[derive(Component, Replicate, Deref, DerefMut)]
/// A percent between 0 and 100
pub struct Luck(pub Property<u8>);

#[derive(Serde, Clone, PartialEq)]
pub struct Meter {
    pub cap: u16,
    pub current: u16,
}

impl Meter {
    pub fn new(cap: u16) -> Self {
        Self { cap, current: 0 }
    }
}
