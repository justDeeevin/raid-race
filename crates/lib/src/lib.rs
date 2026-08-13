pub mod component;
pub mod input;
pub mod player;
pub mod scene;

use avian3d::{
    dynamics::solver::islands::{IslandPlugin, IslandSleepingPlugin},
    interpolation::PhysicsInterpolationPlugin,
    physics_transform::PhysicsTransformPlugin,
    PhysicsPlugins,
};
use bevy::{
    app::{App, PluginGroup, Startup},
    scene::SpawnListSystem,
};
use component::alive::{player::*, status::*, *};
use input::{Jump, Look, Walk};
use lightyear::{
    avian3d::plugin::LightyearAvianPlugin,
    input::bei::prelude::InputPlugin,
    prelude::{input::InputRegistryExt, AppComponentExt},
};
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

pub const TICK_PERIOD: Duration = Duration::from_nanos(7812500); // 128 Hz
pub const SERVER_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
pub const GAME_PORT: u16 = 5000;
pub const AUTH_PORT: u16 = 4000;

pub fn plugin(app: &mut App) {
    macro_rules! replicate {
        ($($cmp:ty),* $(,)?) => {
            $(app.component::<$cmp>().replicate();)*
        }
    }

    replicate!(
        Player, Id, Health, Defense, Mana, Dps, Agility, Cdr, Luck, Pitch, Poison, DefenseUp,
        DefenseUp, DpsUp, DpsDown
    );

    app.add_plugins(InputPlugin::<Player>::default())
        .add_plugins(InputPlugin::<CanJump>::default())
        .register_input_action::<Walk>()
        .register_input_action::<Jump>()
        .register_input_action::<Look>();

    app.add_plugins((
        LightyearAvianPlugin::default(),
        PhysicsPlugins::default()
            .build()
            .disable::<PhysicsTransformPlugin>()
            .disable::<PhysicsInterpolationPlugin>()
            .disable::<IslandPlugin>()
            .disable::<IslandSleepingPlugin>(),
    ));

    app.add_systems(Startup, scene::test.spawn());
}
