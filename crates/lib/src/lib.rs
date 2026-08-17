pub mod component;
pub mod event;
pub mod input;
pub mod player;
pub mod scene;

use avian3d::{
    PhysicsPlugins,
    dynamics::solver::islands::{IslandPlugin, IslandSleepingPlugin},
    interpolation::PhysicsInterpolationPlugin,
    physics_transform::PhysicsTransformPlugin,
};
use bevy::{
    app::{App, PluginGroup, Startup},
    scene::SpawnListSystem,
};
use component::alive::{player::*, status::*, *};
use input::*;
use lightyear::{
    avian3d::plugin::LightyearAvianPlugin,
    prediction::registry::PredictionBuilderExt,
    prelude::{
        AppComponentExt,
        input::{InputRegistryExt, bei::InputPlugin},
    },
};
use player::character::Cooldowns;
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
        Player,
        Id,
        AttackTimer,
        Health,
        Defense,
        Mana,
        Dps,
        Agility,
        Cdr,
        Luck,
        Pitch,
        Poison,
        DefenseUp,
        DefenseUp,
        DpsUp,
        DpsDown,
    );

    app.component::<Cooldowns>().replicate_once();

    app.component::<Pitch>().predict();

    app.add_plugins(InputPlugin::<Player>::default())
        .register_input_action::<Walk>()
        .register_input_action::<Jump>()
        .register_input_action::<Look>()
        .register_input_action::<Attack>()
        .register_input_action::<Ability<1>>()
        .register_input_action::<Ability<2>>()
        .register_input_action::<Ability<3>>()
        .register_input_action::<Ability<4>>()
        .register_input_action::<Ability<5>>();

    app.add_plugins(player::plugin);

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
