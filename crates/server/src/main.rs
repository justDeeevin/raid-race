mod component;
mod plugin;
mod resource;
mod system;

use avian3d::PhysicsPlugins;
use bevy::{
    MinimalPlugins,
    app::{App, Startup},
    asset::AssetPlugin,
    log::LogPlugin,
    mesh::MeshPlugin,
    pbr::{MaterialPlugin, StandardMaterial},
    scene::{ScenePlugin, SpawnListSystem},
};
use raid_race_lib::scene;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            MeshPlugin,
            ScenePlugin,
            MaterialPlugin::<StandardMaterial>::default(),
            PhysicsPlugins::default(),
            LogPlugin::default(),
            plugin::server,
            plugin::player,
            // plugin::status,
        ))
        .add_systems(Startup, scene::test.spawn())
        .run();
}
