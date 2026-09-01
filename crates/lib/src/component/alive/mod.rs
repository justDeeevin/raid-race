pub mod player;
pub mod status;

use std::ops::AddAssign;

use avian3d::math::Scalar;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Deref, Clone, Copy)]
#[component(immutable)]
/// Unique identifier for living entities.
pub struct Id(pub u64);

#[derive(Component, Serialize, Deserialize, Deref, DerefMut, Clone, Default)]
/// # Changing health
///
/// Generally, healing should be done with the [`AddAssign`] impl (e.g. `health += 10`), which
/// prevents overheal. Damage should be done with the [`damage`](Self::damage) method, which handles
/// defense considerations.
///
/// Both of these _can_ be sidestepped by directly accessing the inner [`Meter`], but this should be
/// avoided unless strictly necessary (e.g. true damage or overheal).
pub struct Health(pub Meter);

impl Health {
    pub fn new(cap: u16) -> Self {
        Self(Meter::new(cap))
    }

    pub fn damage(&mut self, damage: u16, defense: i16) {
        self.current = self.current.saturating_sub(
            (damage as f32 * (1.0 - (Defense::DELTA * defense as f32)).max(0.0)) as u16,
        );
    }
}

impl AddAssign<u16> for Health {
    fn add_assign(&mut self, rhs: u16) {
        self.current = self.cap.min(self.current + rhs);
    }
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Mana(pub Meter);

impl AddAssign<u16> for Mana {
    fn add_assign(&mut self, rhs: u16) {
        self.current = self.cap.min(self.current + rhs);
    }
}

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
pub struct Agility(pub i16);

impl Agility {
    pub const SCALER: Scalar = 0.1;
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
/// Cooldown reduction
pub struct Cdr(pub u16);

impl Cdr {
    /// Returns reduction factor for cooldowns.
    ///
    /// The value increases rationally with an asymptote at the max reduction (see function body for
    /// const value).
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

impl Defense {
    pub const DELTA: f32 = 0.1;
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
/// A percent between 0 and 100.
pub struct Luck(pub u8);

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Meter {
    pub cap: u16,
    pub current: u16,
}

impl Meter {
    pub fn new(cap: u16) -> Self {
        Self { cap, current: cap }
    }
}
