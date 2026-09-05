mod cli;
mod system;

use bevy::{prelude::*, window::ExitCondition};
use raid_race_lib::component::alive::Id;
use system::*;

fn main() {
    let args = cli::parse();

    let plugins = if !args.visual {
        DefaultPlugins.set(WindowPlugin {
            primary_window: None,
            exit_condition: ExitCondition::DontExit,
            ..default()
        })
    } else {
        DefaultPlugins.build()
    };

    let mut app = App::default();

    app.add_plugins((plugins, server::plugin, console::plugin, player::plugin))
        .init_resource::<Ids>();

    if args.visual {
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Dir3::Y),
            ));
        });
    }

    app.insert_resource(args).run();
}

#[derive(Default, Resource)]
/// Generates unique ids for entities.
///
/// Just ascending.
struct Ids(u64);

impl Ids {
    /// Generate a new ID.
    pub fn get(&mut self) -> Id {
        self.0 += 1;
        Id(self.0 - 1)
    }
}
