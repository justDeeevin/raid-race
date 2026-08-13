mod plugin;
mod system;

use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup},
    ecs::resource::Resource,
    window::{ExitCondition, WindowPlugin},
};
use raid_race_lib::component::alive::Id;

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
            plugin::status,
            plugin::console,
        ))
        .init_resource::<Ids>()
        .run();
}

#[derive(Default, Resource)]
/// Generates unique ids for entities
///
/// Just ascending
struct Ids(u64);

impl Ids {
    /// Generate a new ID
    pub fn get(&mut self) -> Id {
        self.0 += 1;
        Id(self.0 - 1)
    }
}
