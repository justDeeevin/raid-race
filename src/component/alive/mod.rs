pub mod status;

use bevy::ecs::component::Component;

#[derive(Component)]
pub struct Health(pub u16);

#[derive(Component)]
pub struct Mana(pub u16);

#[derive(Component)]
pub struct Dps(pub u16);

#[derive(Component)]
/// A percent between 0 and 100
pub struct Luck(pub u8);

#[derive(Component)]
pub struct Agility(pub u8);

#[derive(Component)]
pub struct Cdr(u16);

impl Cdr {
    pub fn new(cdr: u16) -> Self {
        Self(cdr)
    }

    pub fn divisor(&self) -> u16 {
        // some asymptotic function
        self.0 + 1
    }
}
