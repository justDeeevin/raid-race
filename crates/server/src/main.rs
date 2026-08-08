mod plugin;
mod system;

use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup},
    window::{ExitCondition, WindowPlugin},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..Default::default()
            }),
            plugin::server,
            plugin::player,
            // plugin::status,
        ))
        .run();
}
