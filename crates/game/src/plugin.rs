use crate::{
    resource::{InputState, Inputs, Looking, Me},
    system::{
        client, player,
        ui::hud::{self, health_bar, mana_bar},
    },
};
use bevy::{
    app::{App, Startup, Update},
    ecs::schedule::{
        IntoScheduleConfigs, SystemCondition,
        common_conditions::{on_message, resource_exists},
    },
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput},
};
use naia_bevy_client::{
    ClientConfig, DefaultClientTag, Plugin as ClientPlugin,
    events::{ClientTickEvent, DespawnEntityEvent, SpawnEntityEvent},
};
use raid_race_lib::protocol;

pub fn hud(app: &mut App) {
    app.add_systems(Update, (health_bar, mana_bar))
        .add_observer(hud::remove_poison);
}

pub fn client(app: &mut App) {
    app.add_plugins(ClientPlugin::<DefaultClientTag>::new(
        ClientConfig::default(),
        protocol(),
    ))
    .add_systems(Startup, client::connect)
    .add_systems(
        Update,
        client::tick.run_if(on_message::<ClientTickEvent<DefaultClientTag>>),
    )
    .add_observer(client::disconnect);
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
