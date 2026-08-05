use bevy::ecs::{
    entity::Entity,
    event::Event,
    message::MessageReader,
    query::With,
    system::{Commands, Query},
};
use bevy_console::{
    ConsoleCommand,
    clap::{self, Parser},
};
use naia_bevy_client::{Client, DefaultClientTag, events::ClientTickEvent, transport::webrtc};
use naia_client::ConnectionStatus;
use raid_race_lib::message::Auth;

use crate::{component::OrbitCamera, resource::Me, system::ui::hud::HudRoot};

#[derive(Event)]
pub struct Tick(pub naia_bevy_client::Tick);

#[derive(Parser, ConsoleCommand)]
#[command(name = "connect")]
/// Connect to a server
pub struct ConnectCommand {
    #[arg()]
    /// The address of the server.
    ///
    /// This does not require a protocol prefix or a port number, but will accept them if
    /// specified.
    pub address: String,
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "disconnect")]
/// Disconnect from the current server
pub struct DisconnectCommand;

pub fn connect(mut client: Client<DefaultClientTag>, address: impl AsRef<str>) {
    client.auth(Auth);
    client.connect(webrtc::Socket::new(
        &make_url(address),
        client.socket_config(),
    ));
}

fn make_url(address: impl AsRef<str>) -> String {
    let mut url = address.as_ref().to_string();

    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("http://{url}");
    }

    if url
        .split(':')
        .next_back()
        .is_none_or(|s| s.parse::<u16>().is_err())
    {
        url += ":14191";
    }

    url
}

pub fn connect_command(mut cmd: ConsoleCommand<ConnectCommand>, client: Client<DefaultClientTag>) {
    if let Some(Ok(cmd)) = cmd.take() {
        connect(client, cmd.address);
    }
}

pub fn tick(mut ticks: MessageReader<ClientTickEvent<DefaultClientTag>>, mut commands: Commands) {
    for event in ticks.read() {
        commands.trigger(Tick(event.tick))
    }
}

pub fn disconnect(
    mut client: Client<DefaultClientTag>,
    hud: Query<Entity, With<HudRoot>>,
    camera: Query<Entity, With<OrbitCamera>>,
    mut commands: Commands,
) -> bool {
    if client.connection_status() != ConnectionStatus::Connected {
        return false;
    };

    if let Ok(entity) = hud.single() {
        commands.entity(entity).despawn();
    }
    for entity in camera {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Me>();

    client.disconnect();

    true
}

pub fn disconnect_command(
    mut cmd: ConsoleCommand<DisconnectCommand>,
    client: Client<DefaultClientTag>,
    hud: Query<Entity, With<HudRoot>>,
    camera: Query<Entity, With<OrbitCamera>>,
    commands: Commands,
) {
    let Some(Ok(_)) = cmd.take() else {
        return;
    };

    if !disconnect(client, hud, camera, commands) {
        cmd.reply_failed("not currently connected");
    }
}
