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

#[derive(Component, Deref, DerefMut)]
pub struct Cdr(pub u16);

impl Cdr {
    pub fn divisor(&self) -> u16 {
        // some asymptotic function
        self.0 + 1
    }
}
