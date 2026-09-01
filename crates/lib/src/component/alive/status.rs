use super::{Cdr, Dps};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{num::NonZero, time::Duration};

#[derive(Component, Serialize, Deserialize)]
pub struct Poison {
    source: Entity,
    pub tick: Timer,
    pub total: Timer,
}

impl Poison {
    pub fn new(source: Entity, cdr: &Cdr, duration: Duration) -> Self {
        const BASE_POISON_PERIOD_SECS: f32 = 2.0;

        Self {
            source,
            tick: Timer::from_seconds(BASE_POISON_PERIOD_SECS * cdr.scaler(), TimerMode::Repeating),
            total: Timer::new(duration, TimerMode::Once),
        }
    }

    pub fn tick_damage(&self, damagers: Query<&Dps>) -> u16 {
        let Ok(Dps(dps)) = damagers.get(self.source) else {
            return 0;
        };

        const BASE_POISON_DPS_DIVISOR: u16 = 20;

        (*dps / BASE_POISON_DPS_DIVISOR) * self.tick.duration().as_secs() as u16
    }
}

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct DefenseUp(pub StackableStatusEffect);
#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct DefenseDown(pub StackableStatusEffect);

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct DpsUp(pub StackableStatusEffect);
#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct DpsDown(pub StackableStatusEffect);

#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct AgilityUp(pub StackableStatusEffect);
#[derive(Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct AgilityDown(pub StackableStatusEffect);

#[derive(Component)]
// TODO:
pub struct Blind(pub Timer);

#[derive(Serialize, Deserialize)]
pub struct StackableStatusEffect {
    pub stacks: NonZero<u8>,
    pub timer: Timer,
}

impl StackableStatusEffect {
    pub fn new(stacks: NonZero<u8>, duration: Duration) -> Self {
        Self {
            stacks,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }
}
