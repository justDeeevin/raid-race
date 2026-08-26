mod system;

use bevy::{prelude::*, window::ExitCondition};
use raid_race_lib::component::alive::Id;
use system::*;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..Default::default()
            }),
            server::plugin,
            status::plugin,
            console::plugin,
            player::plugin,
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
