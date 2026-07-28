mod component;
mod system;

use crate::component::alive::{Cdr, Dps, Health, status::Poison};
use bevy::{
    DefaultPlugins,
    app::{Startup, Update},
    ecs::system::Commands,
    time::Time,
};
use std::time::Duration;

fn main() {
    bevy::app::App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, system::status::poison)
        .init_resource::<Time>()
        .run();
}

fn startup(mut commands: Commands) {
    let mut entity = commands.spawn((Health(100), Dps(100)));
    entity.insert(Poison::new(
        entity.id(),
        &Cdr::new(0),
        Duration::from_secs(10),
    ));
}
