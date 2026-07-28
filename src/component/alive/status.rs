use super::{Cdr, Dps};
use bevy::{
    ecs::{component::Component, entity::Entity, system::Query},
    time::{Timer, TimerMode},
};
use std::{num::NonZero, time::Duration};

#[derive(Component)]
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
            tick: Timer::from_seconds(
                BASE_POISON_PERIOD_SECS / cdr.divisor() as f32,
                TimerMode::Repeating,
            ),
            total: Timer::new(duration, TimerMode::Once),
        }
    }
}

impl Poison {
    pub fn tick_damage(&self, damagers: Query<&Dps>) -> u16 {
        let Ok(Dps(dps)) = damagers.get(self.source) else {
            return 0;
        };

        const BASE_POISON_DPS_DIVISOR: u16 = 20;

        (*dps / BASE_POISON_DPS_DIVISOR) * self.tick.duration().as_secs() as u16
    }
}

#[derive(Component)]
#[component(immutable)]
pub struct DefenseUp(pub StackableStatusEffect);
#[derive(Component)]
#[component(immutable)]
pub struct DefenseDown(pub StackableStatusEffect);

#[derive(Component)]
#[component(immutable)]
pub struct DpsUp(pub StackableStatusEffect);
#[derive(Component)]
#[component(immutable)]
pub struct DpsDown(pub StackableStatusEffect);

#[derive(Component)]
#[component(immutable)]
pub struct Blind(pub Timer);

pub struct StackableStatusEffect {
    pub stacks: NonZero<u8>,
    pub timer: Timer,
}
