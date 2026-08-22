use async_net::TcpStream;
use bevy::{
    asset::{AsyncReadExt, AsyncWriteExt},
    ecs::{
        entity::Entity,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    prelude::{Deref, DerefMut},
    tasks::{IoTaskPool, Task, block_on, poll_once},
};
use bevy_console::{
    ConsoleCommand,
    clap::{self, Parser},
};
use lightyear::{
    connection::client::{Client, Connect, Connected, Disconnect},
    netcode::{
        CONNECT_TOKEN_BYTES, ConnectToken, NetcodeClient, auth::Authentication,
        client_plugin::NetcodeConfig,
    },
    prelude::{LocalAddr, PeerAddr, PingManager, ReplicationReceiver},
    webtransport::client::WebTransportClientIo,
};
use raid_race_lib::{AUTH_PORT, GAME_PORT};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Parser, ConsoleCommand)]
#[command(name = "connect")]
/// Connect to a server
pub struct ConnectCommand {
    #[arg()]
    /// The address of the server.
    ///
    /// This does not require a port number, but will accept one if
    /// specified.
    pub address: String,
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "disconnect")]
/// Disconnect from the current server
pub struct DisconnectCommand;

#[derive(Resource, Deref, DerefMut)]
pub struct AuthServer(IpAddr);

impl Default for AuthServer {
    fn default() -> Self {
        Self(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct TokenTask(Task<ConnectToken>);

pub fn connect(mut commands: Commands, game_server: SocketAddr, auth_server: IpAddr) {
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

// TODO: steam
async fn get_token(game_server: IpAddr, auth_server: IpAddr) -> ConnectToken {
    tracing::info!("fetching auth token");
    let mut stream = TcpStream::connect(SocketAddr::new(auth_server, AUTH_PORT))
        .await
        .expect("failed to connect to auth server");

    match game_server {
        IpAddr::V4(addr) => {
            stream
                .write_all([4].as_slice())
                .await
                .expect("failed to send ip version");
            stream
                .write_all(&addr.octets())
                .await
                .expect("failed to send game server ip");
        }
        IpAddr::V6(addr) => {
            stream
                .write_all([6].as_slice())
                .await
                .expect("failed to send ip version");
            stream
                .write_all(&addr.octets())
                .await
                .expect("failed to send game server ip");
        }
    }

    let mut buffer = [0_u8; CONNECT_TOKEN_BYTES];

    stream
        .read_exact(&mut buffer)
        .await
        .expect("failed to read connect token");

    ConnectToken::try_from_bytes(&buffer).expect("failed to parse connect token from server")
}

pub fn wait_for_token(
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

pub fn connect_command(
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

pub fn disconnect_command(
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
