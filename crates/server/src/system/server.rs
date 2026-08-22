use super::entity;
use crate::Ids;
use async_lock::RwLock;
use async_net::TcpListener;
use avian3d::parry::utils::hashset::HashSet;
use bevy::{
    asset::{AsyncReadExt, AsyncWriteExt},
    ecs::{
        lifecycle::Add,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    prelude::{Deref, DerefMut},
    tasks::IoTaskPool,
};
use lightyear::{
    connection::{
        client::{Connected, Disconnected},
        client_of::ClientOf,
        server::Start,
    },
    core::id::{PeerId, RemoteId},
    link::server::LinkOf,
    netcode::{ConnectToken, Key, NetcodeServer, server_plugin::NetcodeConfig},
    prelude::{Identity, LocalAddr, ReplicationSender},
    webtransport::server::WebTransportServerIo,
};
use raid_race_lib::{AUTH_PORT, GAME_PORT, SERVER_ADDR};
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    sync::Arc,
};
use tracing::info;
use wtransport::tls::{Certificate, CertificateChain, PrivateKey};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ClientIds(Arc<RwLock<HashSet<u64>>>);

// VERSION 0
const PROTOCOL_ID: u64 = 0;
const PRIVATE_KEY: Key = {
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

pub fn serve(mut commands: Commands, ids: Res<ClientIds>) {
    let entity = commands
        .spawn((
            NetcodeServer::new(
                NetcodeConfig::default()
                    .with_protocol_id(PROTOCOL_ID)
                    .with_key(PRIVATE_KEY),
            ),
            LocalAddr(SocketAddr::new(SERVER_ADDR, GAME_PORT)),
            WebTransportServerIo {
                certificate: Identity::new(
                    CertificateChain::single(
                        #[allow(clippy::unwrap_used, reason = "shouldn't fail")]
                        Certificate::from_der(include_bytes!("../../cert.der").into()).unwrap(),
                    ),
                    PrivateKey::from_der_pkcs8(include_bytes!("../../key.der").into()),
                ),
            },
        ))
        .id();

    commands.trigger(Start { entity });

    IoTaskPool::get().spawn(auth_server(ids.clone())).detach();
}

async fn auth_server(ids: Arc<RwLock<HashSet<u64>>>) {
    let listener = TcpListener::bind(SocketAddr::new(SERVER_ADDR, AUTH_PORT))
        .await
        .expect("failed to start auth server");

    info!("started auth server");

    loop {
        let (mut stream, client_addr) = listener
            .accept()
            .await
            .expect("failed to accept auth connection");

        // TODO: consideration for remote servers
        let id = loop {
            let out = rand::random();
            if !ids.read().await.contains(&out) {
                break out;
            }
        };

        let mut version = [0];
        stream
            .read_exact(&mut version)
            .await
            .expect("failed to read ip version");

        let server_addr = match version[0] {
            4 => {
                let mut ip = [0; 4];
                stream
                    .read_exact(&mut ip)
                    .await
                    .expect("failed to read ipv4 address");
                IpAddr::V4(Ipv4Addr::from_octets(ip))
            }
            6 => {
                let mut ip = [0; 16];
                stream
                    .read_exact(&mut ip)
                    .await
                    .expect("failed to read ipv6 address");
                IpAddr::V6(Ipv6Addr::from_octets(ip))
            }
            v => {
                tracing::error!(%client_addr, v, "invalid ip version provided");
                continue;
            }
        };

        let token = ConnectToken::build(
            SocketAddr::new(server_addr, GAME_PORT),
            PROTOCOL_ID,
            id,
            PRIVATE_KEY,
        )
        .generate()
        .expect("failed to generate token");

        let serialized_token = token.try_into_bytes().expect("failed to serialize token");

        stream
            .write_all(&serialized_token)
            .await
            .expect("failed to send token to client");

        info!(%client_addr, %server_addr, id, "sent token to client");
    }
}

pub fn start_join(event: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(event.entity).insert(ReplicationSender);
}

pub fn join(
    event: On<Add, Connected>,
    id: Query<&RemoteId, With<ClientOf>>,
    client_ids: Res<ClientIds>,
    mut ids: ResMut<Ids>,
    mut commands: Commands,
) {
    let Ok(RemoteId(id)) = id.get(event.entity) else {
        return;
    };

    entity::player(100, 40, 100)
        .build()
        .spawn(&mut commands, *id, &mut ids, event.entity);

    if let PeerId::Netcode(id) = id {
        client_ids.write_blocking().insert(*id);
    }
}

pub fn leave(
    event: On<Add, Disconnected>,
    id: Query<&RemoteId, With<ClientOf>>,
    ids: Res<ClientIds>,
) {
    if let Ok(RemoteId(PeerId::Netcode(id))) = id.get(event.entity) {
        ids.write_blocking().remove(id);
    };
}
