#[cfg(feature = "server")]
use super::Defense;
#[cfg(feature = "server")]
use bevy::ecs::component::Mutable;
#[cfg(any(feature = "client", feature = "server"))]
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
#[cfg(feature = "server")]
use std::ops::{AddAssign, Deref, DerefMut, SubAssign};

use super::{Cdr, Dps};
use bevy::{
    ecs::{component::Component, entity::Entity, event::Event, system::Query},
    prelude::{Deref, DerefMut},
    time::{Timer, TimerMode},
};
use std::{num::NonZero, time::Duration};

#[derive(Component)]
#[cfg_attr(feature = "client", component(on_remove = Self::on_remove))]
pub struct Poison {
    source: Entity,
    pub tick: Timer,
    pub total: Timer,
}

#[derive(Event)]
pub struct PoisonRemoved {
    pub from: Entity,
}

impl Poison {
    pub fn new(source: Entity, cdr: &Cdr, duration: Duration) -> Self {
        const BASE_POISON_PERIOD_SECS: f32 = 2.0;

        Self {
            source,
            tick: Timer::from_seconds(BASE_POISON_PERIOD_SECS * cdr.scale(), TimerMode::Repeating),
            total: Timer::new(duration, TimerMode::Once),
        }
    }

    pub fn tick_damage(&self, damagers: Query<&Dps>) -> u16 {
        let Ok(Dps(dps)) = damagers.get(self.source) else {
            return 0;
        };

        const BASE_POISON_DPS_DIVISOR: u16 = 20;

        (**dps / BASE_POISON_DPS_DIVISOR) * self.tick.duration().as_secs() as u16
    }

    #[cfg(feature = "client")]
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        world.trigger(PoisonRemoved {
            from: context.entity,
        });
    }
}

#[derive(Component, Deref, DerefMut)]
#[cfg_attr(feature = "server", component(on_add = on_add::<false, Self, Defense>, on_remove = on_remove::<false, Self, Defense>))]
pub struct DefenseUp(pub StackableStatusEffect);
#[derive(Component, Deref, DerefMut)]
#[cfg_attr(feature = "server", component(on_add = on_add::<true, Self, Defense>, on_remove = on_remove::<true, Self, Defense>))]
pub struct DefenseDown(pub StackableStatusEffect);

#[derive(Component, Deref, DerefMut)]
#[cfg_attr(feature = "server", component(on_add = on_add::<false, Self, Defense>, on_remove = on_remove::<false, Self, Defense>))]
pub struct DpsUp(pub StackableStatusEffect);
#[derive(Component, Deref, DerefMut)]
#[cfg_attr(feature = "server", component(on_add = on_add::<true, Self, Defense>, on_remove = on_remove::<true, Self, Defense>))]
pub struct DpsDown(pub StackableStatusEffect);

#[derive(Component)]
#[component(immutable)]
pub struct Blind(pub Timer);

pub struct StackableStatusEffect {
    pub stacks: NonZero<u8>,
    timer: Timer,
}

impl StackableStatusEffect {
    pub fn new(stacks: NonZero<u8>, duration: Duration) -> Self {
        Self {
            stacks,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, time: Duration) {
        self.timer.tick(time);
    }

    pub fn timer(&self) -> &Timer {
        &self.timer
    }
}

#[cfg(feature = "server")]
fn on_add<
    const DEBUFF: bool,
    T: Component + std::ops::Deref<Target = StackableStatusEffect>,
    Target: Component<Mutability = Mutable> + DerefMut,
>(
    mut world: DeferredWorld,
    context: HookContext,
) where
    Target::Target: DerefMut,
    <Target::Target as Deref>::Target: Sized + From<u8> + AddAssign + SubAssign,
{
    #[allow(
        clippy::unwrap_used,
        reason = "on_add is always called after T is added"
    )]
    let delta = world.get::<T>(context.entity).unwrap().stacks.get().into();
    let mut target = world
        .get_mut::<Target>(context.entity)
        .expect("debuff target not found");

    if DEBUFF {
        // possible underflow
        ***target -= delta;
    } else {
        ***target += delta;
    }
}

#[cfg(feature = "server")]
fn on_remove<
    const DEBUFF: bool,
    T: Component + std::ops::Deref<Target = StackableStatusEffect>,
    Target: Component<Mutability = Mutable> + DerefMut,
>(
    mut world: DeferredWorld,
    context: HookContext,
) where
    Target::Target: DerefMut,
    <Target::Target as Deref>::Target: Sized + From<u8> + AddAssign + SubAssign,
{
    #[allow(
        clippy::unwrap_used,
        reason = "on_remove is always called right before T is removed"
    )]
    let delta = world.get::<T>(context.entity).unwrap().stacks.get().into();
    let mut target = world
        .get_mut::<Target>(context.entity)
        .expect("debuff target not found");

    if DEBUFF {
        ***target += delta;
    } else {
        ***target -= delta;
    }
}
