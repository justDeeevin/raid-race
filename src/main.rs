#![deny(clippy::unwrap_used)]

mod component;
mod scene;
mod system;

use avian3d::PhysicsPlugins;
use bevy::{
    DefaultPlugins,
    app::{PostStartup, Startup, Update},
    scene::SpawnListSystem,
};
use component::alive::status::{DefenseDown, DefenseUp, DpsDown, DpsUp};
use system::status::stat_change;

fn main() {
    bevy::app::App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .add_systems(Startup, (scene::main.spawn(), system::spawn::player))
        .add_systems(PostStartup, (system::spawn::hud, setup::setup))
        .add_systems(
            Update,
            (
                system::status::poison,
                stat_change::<DefenseUp>,
                stat_change::<DefenseDown>,
                stat_change::<DpsUp>,
                stat_change::<DpsDown>,
                system::player::movement,
                system::ui::health_bar,
            ),
        )
        .run();
}

#[allow(clippy::unwrap_used, reason = "testing stuff")]
mod setup {
    use std::time::Duration;

    use bevy::ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query},
    };

    use crate::component::alive::{Cdr, player::Player, status::Poison};

    pub fn setup(mut commands: Commands, player: Query<(Entity, &Cdr), With<Player>>) {
        let (player, cdr) = player.single().unwrap();
        commands
            .entity(player)
            .insert(Poison::new(player, cdr, Duration::from_secs(10)));
    }
}
