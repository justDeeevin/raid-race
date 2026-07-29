#![deny(clippy::unwrap_used)]

mod component;
mod system;

use bevy::{
    DefaultPlugins,
    app::{Startup, Update},
    ecs::system::Commands,
    time::Time,
};
use component::alive::{
    Cdr, Defense, Dps, Health,
    status::{DefenseDown, DefenseUp, DpsDown, DpsUp, Poison, StackableStatusEffect},
};
use std::{num::NonZero, time::Duration};
use system::status::stat_change;

fn main() {
    bevy::app::App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                system::status::poison,
                stat_change::<DefenseUp>,
                stat_change::<DefenseDown>,
                stat_change::<DpsUp>,
                stat_change::<DpsDown>,
            ),
        )
        .init_resource::<Time>()
        .run();
}

fn startup(mut commands: Commands) {
    let mut entity = commands.spawn((
        Health(100),
        Dps(100),
        Defense(10),
        DefenseDown(StackableStatusEffect::new(
            #[allow(clippy::unwrap_used, reason = "statically safe")]
            NonZero::new(1).unwrap(),
            Duration::from_secs(5),
        )),
    ));
    entity.insert(Poison::new(
        entity.id(),
        &Cdr::new(0),
        Duration::from_secs(10),
    ));
}
