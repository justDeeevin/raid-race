mod component;
mod plugin;
mod resource;
mod system;

use avian3d::PhysicsPlugins;
use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Startup, Update},
    ecs::{
        entity::Entity,
        event::Event,
        observer::On,
        query::With,
        schedule::{IntoScheduleConfigs, common_conditions::on_message},
        system::{Commands, Query},
    },
    log::LogPlugin,
    scene::SpawnListSystem,
    window::{PrimaryWindow, WindowCloseRequested, WindowPlugin},
};
use bevy_console::{ConsolePlugin, make_layer};
use naia_bevy_client::{AppRegisterComponentEvents, DefaultClientTag};
use raid_race_lib::{
    component::alive::{Agility, Cdr, Defense, Dps, Health, Luck, Mana, player::SimSync},
    scene,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    close_when_requested: false,
                    ..Default::default()
                })
                .set(LogPlugin {
                    custom_layer: make_layer,
                    ..Default::default()
                }),
            ConsolePlugin,
            PhysicsPlugins::default(),
            plugin::client,
            plugin::player,
            plugin::hud,
            // avian3d::debug_render::PhysicsDebugPlugin,
        ))
        .add_component_events::<DefaultClientTag, SimSync>()
        .add_component_events::<DefaultClientTag, Health>()
        .add_component_events::<DefaultClientTag, Mana>()
        .add_component_events::<DefaultClientTag, Luck>()
        .add_component_events::<DefaultClientTag, Agility>()
        .add_component_events::<DefaultClientTag, Defense>()
        .add_component_events::<DefaultClientTag, Dps>()
        .add_component_events::<DefaultClientTag, Cdr>()
        .add_systems(Startup, scene::test.spawn())
        .add_systems(
            Update,
            (|mut commands: Commands| commands.trigger(Quit))
                .run_if(on_message::<WindowCloseRequested>),
        )
        .add_observer(
            |_: On<Quit>, win: Query<Entity, With<PrimaryWindow>>, mut commands: Commands| {
                #[allow(
                    clippy::unwrap_used,
                    reason = "there's only one primary window and it still exists by this point"
                )]
                commands.entity(win.single().unwrap()).despawn()
            },
        )
        .run();
}

#[derive(Event)]
pub struct Quit;
