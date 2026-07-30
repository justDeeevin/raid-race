#![deny(clippy::unwrap_used)]

mod component;
mod plugin;
mod resource;
mod scene;
mod system;

use avian3d::PhysicsPlugins;
use bevy::{
    DefaultPlugins,
    app::{App, PostStartup, Startup, Update},
    scene::SpawnListSystem,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            plugin::movement,
            plugin::status,
            plugin::hud,
        ))
        .add_systems(Startup, (scene::main.spawn(), system::spawn::player))
        .add_systems(Update, system::player::camera)
        .add_systems(PostStartup, test::setup)
        .run();
}

#[allow(clippy::unwrap_used, reason = "testing stuff")]
mod test {
    use crate::component::alive::{Cdr, player::Player, status::Poison};
    use bevy::ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query},
    };
    use std::time::Duration;

    pub fn setup(mut commands: Commands, player: Query<(Entity, &Cdr), With<Player>>) {
        let (player, cdr) = player.single().unwrap();
        commands
            .entity(player)
            .insert(Poison::new(player, cdr, Duration::from_secs(10)));
    }
}
