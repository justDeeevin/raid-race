use bevy::prelude::*;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, ValueEnum, Clone, Copy)]
#[clap(rename_all = "snake_case")]
pub enum Weapon {
    #[clap(name = "placeholder")]
    PlaceholderGun,
}

#[derive(Component, Serialize, Deserialize, Deref, DerefMut)]
pub struct HeldWeapon(pub Weapon);
