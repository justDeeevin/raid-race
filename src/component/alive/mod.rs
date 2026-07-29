pub mod player;
pub mod status;

use bevy::{
    ecs::component::Component,
    prelude::{Deref, DerefMut},
};

#[derive(Component, Deref, DerefMut)]
pub struct Health(pub u16);

#[derive(Component, Deref, DerefMut)]
pub struct Defense(pub i16);

#[derive(Component, Deref, DerefMut)]
pub struct Mana(pub u16);

#[derive(Component, Deref, DerefMut)]
pub struct Dps(pub u16);

#[derive(Component, Deref, DerefMut)]
/// A percent between 0 and 100
pub struct Luck(pub u8);

#[derive(Component, Deref, DerefMut)]
pub struct Agility(pub u8);

impl Agility {
    pub const MOVE_SPEED_ADJUST: f64 = 0.1;
}

#[derive(Component, Deref, DerefMut)]
pub struct Cdr(pub u16);

impl Cdr {
    pub fn scale(&self) -> f32 {
        const MAX_REDUCTION: f32 = 0.3;
        const HALFWAY_POINT: f32 = 15.0;

        let x = self.0 as f32;

        // rational function
        let f: f32 = x / (x + HALFWAY_POINT);

        1.0 - (MAX_REDUCTION * f)
    }
}
