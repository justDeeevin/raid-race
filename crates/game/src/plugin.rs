use crate::{
    Quit,
    component::OrbitCamera,
    resource::{InputState, Inputs, Looking, Me},
    system::{
        client::{self, ConnectCommand, DisconnectCommand},
        player,
        ui::hud::{self, HudRoot},
    },
};
use bevy::{
    app::{App, Update},
    ecs::{
        entity::Entity,
        observer::On,
        query::With,
        schedule::{
            IntoScheduleConfigs, SystemCondition,
            common_conditions::{on_message, resource_exists},
        },
        system::{Commands, Query},
    },
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput},
};
use bevy_console::AddConsoleCommand;
use naia_bevy_client::{
    Client, ClientConfig, DefaultClientTag, Plugin as ClientPlugin,
    events::{ClientTickEvent, DespawnEntityEvent, SpawnEntityEvent},
};
use raid_race_lib::protocol;

pub fn hud(app: &mut App) {
    app.add_systems(Update, (hud::health_bar, hud::mana_bar))
        .add_observer(hud::remove_poison);
}

pub fn client(app: &mut App) {
    app.add_plugins(ClientPlugin::<DefaultClientTag>::new(
        ClientConfig::default(),
        protocol(),
    ))
    .add_systems(
        Update,
        client::tick.run_if(on_message::<ClientTickEvent<DefaultClientTag>>),
    )
    .add_observer(
        |_: On<Quit>,
         client: Client<DefaultClientTag>,
         hud: Query<Entity, With<HudRoot>>,
         camera: Query<Entity, With<OrbitCamera>>,
         commands: Commands| {
            client::disconnect(client, hud, camera, commands);
        },
    )
    .add_console_command::<ConnectCommand, _>(client::connect_command)
    .add_console_command::<DisconnectCommand, _>(client::disconnect_command);
}

pub fn player(app: &mut App) {
    app.add_systems(
        Update,
        (
            player::spawn.run_if(on_message::<SpawnEntityEvent<DefaultClientTag>>),
            player::despawn.run_if(on_message::<DespawnEntityEvent<DefaultClientTag>>),
            player::sync_sim,
            player::camera,
            player::read_input.run_if(on_message::<KeyboardInput>),
            player::simulate_input.run_if(resource_exists::<Me>),
            player::grabber
                .run_if(on_message::<KeyboardInput>.or_eager(on_message::<MouseButtonInput>)),
        ),
    )
    .add_observer(player::send_input)
    .init_resource::<InputState>()
    .init_resource::<Inputs>()
    .init_resource::<Looking>();
}
