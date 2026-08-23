use crate::input::Ability;
use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        system::{Commands, Query, Res},
    },
    prelude::{Deref, DerefMut},
    time::{Time, Timer, TimerMode},
};
use bevy_enhanced_input::action::{InputAction, events::Start};
use either::Either;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Abilities<T: AbilityId>(pub [T; 5]);

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Cooldowns(pub [Either<Timer, bool>; 5]);

impl<T: AbilityId> From<&Abilities<T>> for Cooldowns {
    fn from(value: &Abilities<T>) -> Self {
        Self(std::array::from_fn(|i| {
            value[i].cooldown().map_left(|d| {
                let mut out = Timer::new(d, TimerMode::Once);
                out.almost_finish();
                out
            })
        }))
    }
}

pub trait AbilityId {
    fn trigger(&self, entity: Entity, commands: &mut Commands);
    fn cooldown(&self) -> Either<Duration, bool>;
}

fn ability<const N: u8, A: AbilityId + Send + Sync + 'static>(
    event: On<Start<Ability<N>>>,
    mut characters: Query<(&Abilities<A>, &mut Cooldowns)>,
    mut commands: Commands,
) where
    Ability<N>: InputAction,
{
    if let Ok((abilities, mut cooldowns)) = characters.get_mut(event.context)
        && let Some(ability) = abilities.get(N as usize - 1)
    {
        match cooldowns.get_mut(N as usize - 1) {
            Some(Either::Left(timer)) if timer.is_finished() => {
                timer.reset();
                ability.trigger(event.context, &mut commands);
            }
            Some(Either::Right(ready)) if *ready => {
                *ready = false;
                ability.trigger(event.context, &mut commands);
            }
            Some(_) | None => {}
        }
    }
}

pub fn ability_cooldown(cooldowns: Query<&mut Cooldowns>, time: Res<Time>) {
    for mut cooldowns in cooldowns {
        for cooldown in &mut **cooldowns {
            if let Either::Left(timer) = cooldown {
                timer.tick(time.delta());
            }
        }
    }
}

macro_rules! abilities {
    (
        $($ability:ident {
            cast: ($event:ident, $($param:pat_param| $type:ty),* $(,)?) $body:block $(,
            cooldown: $cooldown:expr)? $(,
            ready: $ready:expr)? $(,)?
        }),* $(,
        !Default: [$one:ident, $two:ident, $three:ident, $four:ident, $five:ident$(,)?])? $(,)?
    ) => {
        #[derive(::serde::Serialize, ::serde::Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum AbilityId {$($ability),*}
        $(
            struct $ability;
            impl $ability {
                fn cast($event: ::bevy::ecs::observer::On<$crate::event::Cast::<$ability>>, $($param: $type),*) $body
            }
        )*

        impl $crate::player::character::AbilityId for AbilityId {
            fn trigger(&self, entity: ::bevy::ecs::entity::Entity, commands: &mut ::bevy::ecs::system::Commands) {
                match self {$(
                        Self::$ability => commands.trigger($crate::event::Cast::<$ability>::new(entity))
                ),*}
            }

            fn cooldown(&self) -> ::either::Either<::std::time::Duration, bool> {
                use ::either::Either;
                match self {$(
                        Self::$ability => {
                            $(Either::Left($cooldown))?
                            $(Either::Right($ready))?
                        }
                ),*}
            }
        }

        fn add_ability_systems(app: &mut ::bevy::app::App) {
            use $crate::player::character::ability;

            app.add_observer(ability::<1, AbilityId>)
                .add_observer(ability::<2, AbilityId>)
                .add_observer(ability::<3, AbilityId>)
                .add_observer(ability::<4, AbilityId>)
                .add_observer(ability::<5, AbilityId>);

            ::lightyear::prelude::AppComponentExt::component::<$crate::player::character::Abilities<AbilityId>>(app).replicate();

            $(app.add_observer($ability::cast);)*
        }

        $(
            impl Default for $crate::player::character::Abilities<AbilityId> {
                fn default() -> Self {
                    Self([
                        AbilityId::$one,
                        AbilityId::$two,
                        AbilityId::$three,
                        AbilityId::$four,
                        AbilityId::$five,
                    ])
                }
            }
        )?
    }
}

pub mod warrior;
