use async_net::TcpStream;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, block_on, poll_once},
};
use bevy_console::{
    AddConsoleCommand, ConsoleCommand,
    clap::{self, Parser},
};
use futures::TryFutureExt;
use http_types::Request;
use lightyear::{
    netcode::{ConnectToken, NetcodeClient, client_plugin::NetcodeConfig},
    prelude::{client::ClientPlugins, *},
    webtransport::client::WebTransportClientIo,
};
use raid_race_lib::{AUTH_PORT, GAME_PORT, TICK_PERIOD};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Parser, ConsoleCommand)]
#[command(name = "connect")]
/// Connect to a server
struct ConnectCommand {
    #[arg()]
    /// The address of the server.
    ///
    /// This does not require a port number, but will accept one if
    /// specified.
    address: String,
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "disconnect")]
/// Disconnect from the current server
struct DisconnectCommand;

#[derive(Parser, ConsoleCommand)]
#[command(name = "auth")]
/// Set the address of the auth server
struct AuthCommand {
    #[arg()]
    address: String,
}

#[derive(Resource, Deref, DerefMut)]
struct AuthServer(IpAddr);

impl Default for AuthServer {
    fn default() -> Self {
        Self(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }
}

fn auth_command(mut cmd: ConsoleCommand<AuthCommand>, mut auth_server: ResMut<AuthServer>) {
    let Some(Ok(AuthCommand { address })) = cmd.take() else {
        return;
    };

    if address == "localhost" {
        **auth_server = IpAddr::V4(Ipv4Addr::LOCALHOST);
    } else if let Ok(addr) = address.parse() {
        **auth_server = addr;
    } else {
        cmd.reply_failed("invalid address");
    }
}

#[derive(Resource, Deref, DerefMut)]
struct TokenTask(Task<ConnectToken>);

// TODO: steam
fn connect(mut commands: Commands, game_server: SocketAddr, auth_server: IpAddr) {
    const CLIENT_ADDR: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);

    commands.spawn((
        Client,
        ReplicationReceiver,
        LocalAddr(SocketAddr::new(CLIENT_ADDR, 0)),
        PeerAddr(game_server),
        PingManager::default(),
        WebTransportClientIo {
            certificate_digest: include_str!("../../../server/digest.txt").into(),
            target: None,
        },
    ));

    let task = IoTaskPool::get().spawn(get_token(game_server.ip(), auth_server));
    commands.insert_resource(TokenTask(task))
}

async fn get_token(game_server: IpAddr, auth_server: IpAddr) -> ConnectToken {
    tracing::info!("fetching auth token");

    let stream = TcpStream::connect(SocketAddr::new(auth_server, AUTH_PORT))
        .await
        .expect("failed to connect to auth server");

    let bytes = async_h1::connect(
        stream,
        Request::get(format!("http://{auth_server}:{AUTH_PORT}/{game_server}").as_str()),
    )
    .and_then(async |mut res| res.body_bytes().await)
    .await
    .expect("failed to get auth token");

    ConnectToken::try_from_bytes(&bytes).expect("failed to parse connect token")
}

fn wait_for_token(
    mut task: ResMut<TokenTask>,
    client: Query<Entity, With<Client>>,
    mut commands: Commands,
) {
    if let Some(token) = block_on(poll_once(&mut **task)) {
        let entity = client.single().expect("no client");
        commands.remove_resource::<TokenTask>();

        #[allow(clippy::unwrap_used, reason = "should never fail")]
        {
            commands.entity(entity).insert(
                NetcodeClient::new(Authentication::Token(token), NetcodeConfig::default()).unwrap(),
            );
        }

        commands.trigger(Connect { entity });
    }
}

fn connect_command(
    mut cmd: ConsoleCommand<ConnectCommand>,
    auth_server: Res<AuthServer>,
    commands: Commands,
) {
    let Some(Ok(ConnectCommand { address })) = cmd.take() else {
        return;
    };

    let addr = if address == "localhost" {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), GAME_PORT)
    } else if let Ok(addr) = address.parse() {
        addr
    } else if let Ok(addr) = address.parse::<IpAddr>() {
        SocketAddr::new(addr, GAME_PORT)
    } else {
        cmd.reply_failed("invalid address");
        return;
    };

    connect(commands, addr, **auth_server);
}

fn disconnect_command(
    mut cmd: ConsoleCommand<DisconnectCommand>,
    client: Query<Entity, (With<Client>, With<Connected>)>,
    mut commands: Commands,
) {
    if cmd.take().is_none_or(|r| r.is_err()) {
        return;
    }

    if let Ok(entity) = client.single() {
        commands.trigger(Disconnect { entity });
    } else {
        cmd.reply_failed("not connected");
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        ClientPlugins {
            tick_duration: TICK_PERIOD,
        },
        raid_race_lib::plugin,
    ))
    .add_systems(Update, wait_for_token.run_if(resource_exists::<TokenTask>))
    .add_console_command::<ConnectCommand, _>(connect_command)
    .add_console_command::<DisconnectCommand, _>(disconnect_command)
    .add_console_command::<AuthCommand, _>(auth_command)
    .init_resource::<AuthServer>();
}
