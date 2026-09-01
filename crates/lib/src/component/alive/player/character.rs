use crate::{input::N_ABILITIES, system::player::character::warrior};
use bevy::prelude::*;
use clap::ValueEnum;
use either::Either;
use serde::{Deserialize, Serialize};
use strum::{EnumDiscriminants, EnumString, IntoStaticStr};

pub type Abilities<T> = [T; N_ABILITIES];

#[derive(Component, Serialize, Deserialize)]
pub struct Character {
    pub data: CharacterData,
    pub channel: Option<Timer>,
}

impl Character {
    pub fn warrior(strike_bonus_percent: u8) -> (Self, Abilities<warrior::AbilityId>) {
        use warrior::AbilityId;

        let abilities = [
            AbilityId::Strike,
            AbilityId::Leap,
            AbilityId::Spin,
            AbilityId::Meditate,
            AbilityId::Kick,
        ];

        (
            Self {
                data: CharacterData::Warrior {
                    strike_bonus_percent,
                    abilities,
                    combo_window: None,
                    combo_index: Some(2),
                    strike: false,
                    combo: 0,
                    spin: None,
                    meditate: None,
                    trance: None,
                },
                channel: None,
            },
            abilities,
        )
    }
}

#[derive(Component, Serialize, Deserialize, EnumDiscriminants, IntoStaticStr)]
#[strum_discriminants(derive(EnumString, ValueEnum), name(CharacterName))]
pub enum CharacterData {
    Warrior {
        abilities: Abilities<warrior::AbilityId>,
        // TODO: bonus damage math from level
        strike_bonus_percent: u8,
        combo_window: Option<Timer>,
        combo_index: Option<usize>,
        strike: bool,
        combo: u8,
        spin: Option<(usize, Timer)>,
        meditate: Option<Timer>,
        trance: Option<Timer>,
    },
}

impl CharacterData {
    pub fn ability(&self, slot: usize) -> &dyn AbilityId {
        match self {
            CharacterData::Warrior { abilities, .. } => &abilities[slot - 1],
        }
    }
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct Cooldowns(pub [Either<Timer, bool>; N_ABILITIES]);

impl<T: AbilityId> From<&Abilities<T>> for Cooldowns {
    fn from(value: &Abilities<T>) -> Self {
        Self(std::array::from_fn(|i| value[i].cooldown()))
    }
}

pub trait AbilityId: std::fmt::Display {
    fn trigger(&self, entity: Entity, commands: &mut Commands);
    fn cooldown(&self) -> Either<Timer, bool>;
    fn description(&self) -> String;
}
