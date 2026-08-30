pub mod component;
pub mod event;
pub mod input;
pub mod scene;
pub mod system;

use avian3d::prelude::*;
use bevy::{
    app::{App, PluginGroup, Startup},
    scene::SpawnListSystem,
};
use bevy_blockout::BlockoutPlugin;
use component::alive::{
    player::{character::*, weapon::*, *},
    status::*,
    *,
};
use event::Attacked;
use input::*;
use lightyear::{
    avian3d::plugin::LightyearAvianPlugin,
    netcode::Key,
    prelude::{
        input::{InputRegistryExt, bei::InputPlugin},
        *,
    },
};
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::LazyLock,
    time::Duration,
};
use totp_rs::{Builder, Totp};

pub const TICK_PERIOD: Duration = Duration::from_nanos(7812500); // 128 Hz
pub const SERVER_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
pub const GAME_PORT: u16 = 5000;
pub const AUTH_PORT: u16 = 4000;
pub const ID_PORT: u16 = 4001;

// VERSION 0
pub const PROTOCOL_ID: u64 = 0;
pub const PRIVATE_KEY: Key = {
    const fn hex(char: u8) -> u8 {
        match char {
            b'0'..=b'9' => char - b'0',
            b'a'..=b'f' => char - b'a' + 10,
            b'A'..=b'F' => char - b'A' + 10,
            _ => panic!("invalid hex character"),
        }
    }

    let bytes = env!("RAID_RACE_PRIVATE_KEY").as_bytes();
    assert!(
        bytes.len() == 64,
        "private key must be 32 bytes (64 hex characters)"
    );
    let mut out = [0; 32];
    let mut i = 0;

    while i < 32 {
        let j = i * 2;
        out[i] = (hex(bytes[j]) << 4) + hex(bytes[j + 1]);
        i += 1;
    }

    out
};

pub static TOTP: LazyLock<Totp> = LazyLock::new(|| {
    #[allow(clippy::unwrap_used, reason = "should never fail")]
    Builder::default().with_secret(PRIVATE_KEY).build().unwrap()
});

pub fn plugin(app: &mut App) {
    macro_rules! replicate {
        ($($cmp:ty),* $(,)?) => {
            $(app.component::<$cmp>().replicate();)*
        }
    }

    replicate!(
        Agility,
        Cdr,
        Character,
        Defense,
        DefenseDown,
        DefenseUp,
        Dps,
        DpsDown,
        DpsUp,
        Health,
        HeldWeapon,
        Id,
        Luck,
        Mana,
        Pitch,
        Player,
        Poison,
        Weapon,
    );

    app.component::<Cooldowns>().replicate_once();
    app.component::<AttackCooldown>().replicate_once();
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

    app.add_plugins((system::player::plugin, system::status::plugin));

    app.add_plugins((
        LightyearAvianPlugin::default(),
        PhysicsPlugins::default()
            .build()
            .disable::<PhysicsTransformPlugin>()
            .disable::<PhysicsInterpolationPlugin>()
            .disable::<IslandPlugin>()
            .disable::<IslandSleepingPlugin>(),
    ));

    app.register_message::<Attacked>()
        .add_direction(NetworkDirection::ServerToClient)
        .add_map_entities();
    app.add_channel::<Channel>(ChannelSettings::default())
        .add_direction(NetworkDirection::Bidirectional);

    app.add_plugins(BlockoutPlugin)
        .add_systems(Startup, scene::test.spawn());
}

pub struct Channel;
