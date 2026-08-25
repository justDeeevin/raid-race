mod component;
mod plugin;
mod system;

use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Startup},
    camera::Camera3d,
    ecs::system::Commands,
    log::LogPlugin,
    math::{Dir3, Vec3},
    transform::components::Transform,
};
use bevy_console::ConsolePlugin;
use component::AimCamera;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                custom_layer: bevy_console::make_layer,
                ..Default::default()
            }),
            ConsolePlugin,
            // plugin::inspector,
            plugin::client,
            plugin::player,
            plugin::hud,
            avian3d::debug_render::PhysicsDebugPlugin,
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 100.0, 100.0).looking_at(Vec3::ZERO, Dir3::Y),
                AimCamera,
            ));
        })
        .run();
}
