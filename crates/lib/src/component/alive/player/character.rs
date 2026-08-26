use crate::{input::N_ABILITIES, system::player::character::warrior};
use bevy::{
    ecs::{component::Component, entity::Entity, system::Commands},
    prelude::{Deref, DerefMut},
    time::{Timer, TimerMode},
};
use clap::ValueEnum;
use either::Either;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use strum::{EnumDiscriminants, EnumString};

pub type Abilities<T> = [T; N_ABILITIES];

#[derive(Component, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(EnumString, ValueEnum), name(CharacterName))]
pub enum Character {
    Warrior {
        abilities: Abilities<warrior::AbilityId>,
        strike_bonus_percent: u8,
        combo_window: Option<Timer>,
        combo_slot: Option<usize>,
        strike: bool,
        combo: u8,
        spin_timer: Option<(usize, Timer)>,
    },
}

impl Character {
    pub fn trigger<const N: usize>(&self, entity: Entity, commands: &mut Commands) {
        match self {
            Self::Warrior { abilities, .. } => abilities[N - 1].trigger(entity, commands),
        }
    }

    pub fn warrior(strike_bonus_percent: u8) -> (Self, Abilities<warrior::AbilityId>) {
        use warrior::AbilityId;

        let abilities = [
            AbilityId::Strike,
            AbilityId::StrikeCombo,
            AbilityId::Spin,
            AbilityId::Strike,
            AbilityId::Strike,
        ];

        (
            Self::Warrior {
                strike_bonus_percent,
                abilities,
                combo_window: None,
                combo_slot: Some(2),
                strike: false,
                combo: 0,
                spin_timer: None,
            },
            abilities,
        )
    }
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Cooldowns(pub [Either<Timer, bool>; N_ABILITIES]);

impl<T: AbilityId> From<&Abilities<T>> for Cooldowns {
    fn from(value: &Abilities<T>) -> Self {
        Self(std::array::from_fn(|i| {
            value[i].cooldown().map_left(|d| {
                let mut out = Timer::new(d, TimerMode::Once);
                out.finish();
                out
            })
        }))
    }
}

pub trait AbilityId {
    fn trigger(&self, entity: Entity, commands: &mut Commands);
    fn cooldown(&self) -> Either<Duration, bool>;
}
