#![deny(clippy::unwrap_used)]

mod component;
mod entity;
mod scene;
mod system;

use avian3d::PhysicsPlugins;
use bevy::{
    DefaultPlugins,
    app::{Startup, Update},
    scene::SpawnListSystem,
};
use component::alive::status::{DefenseDown, DefenseUp, DpsDown, DpsUp};
use system::status::stat_change;

fn main() {
    bevy::app::App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, (scene::main.spawn(), system::spawn::player))
        .add_systems(
            Update,
            (
                system::status::poison,
                stat_change::<DefenseUp>,
                stat_change::<DefenseDown>,
                stat_change::<DpsUp>,
                stat_change::<DpsDown>,
                system::player::movement,
            ),
        )
        .run();
}

mod startup {}
