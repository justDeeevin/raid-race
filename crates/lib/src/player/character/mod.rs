use crate::input::Ability;
use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        system::{Commands, Query, Res},
    },
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
};
use bevy_enhanced_input::action::{InputAction, events::Start};
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Abilities<T: AbilityId>([T; 5]);

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Cooldowns(pub [Timer; 5]);

pub trait AbilityId {
    fn trigger(&self, entity: Entity, commands: &mut Commands);
}

fn ability<const N: u8, A: AbilityId + Send + Sync + 'static>(
    event: On<Start<Ability<N>>>,
    mut characters: Query<(&Abilities<A>, &mut Cooldowns)>,
    mut commands: Commands,
) where
    Ability<N>: InputAction,
{
    if let Ok((abilities, mut cooldowns)) = characters.get_mut(event.context)
        && let Some(id) = abilities.get(N as usize - 1)
        && let Some(timer) = cooldowns.get_mut(N as usize - 1)
        && timer.is_finished()
    {
        timer.reset();
        id.trigger(event.context, &mut commands);
    }
}

pub fn ability_cooldown(cooldowns: Query<&mut Cooldowns>, time: Res<Time>) {
    for mut cooldowns in cooldowns {
        for timer in &mut **cooldowns {
            timer.tick(time.delta());
        }
    }
}

macro_rules! abilities {
    ($($ability:ident($event:ident, $($param:pat_param| $type:ty),* $(,)?) $body:block),*, $(!Default: [$one:ident, $two:ident, $three:ident, $four:ident, $five:ident$(,)?])? $(,)?) => {
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        pub enum AbilityId {$($ability),*}
        $(
            struct $ability;
            impl $ability {
                fn cast($event: ::bevy::ecs::observer::On<$crate::event::Cast::<$ability>>, $($param: $type),*) $body
            }
        )*

        impl $crate::player::character::AbilityId for AbilityId {
            fn trigger(&self, entity: ::bevy::ecs::entity::Entity, commands: &mut ::bevy::ecs::system::Commands) {
                match self {
                    $(
                        Self::$ability => commands.trigger($crate::event::Cast::<$ability>::new(entity))
                    ),*
                }
            }
        }

        fn add_ability_systems(app: &mut ::bevy::app::App) {
            use $crate::player::character::ability;

            app
                .add_observer(ability::<1, AbilityId>)
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
