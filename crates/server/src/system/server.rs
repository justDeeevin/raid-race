use super::entity;
use async_lock::RwLock;
use async_net::TcpListener;
use avian3d::parry::utils::hashset::HashSet;
use bevy::{
    asset::AsyncWriteExt,
    ecs::{
        lifecycle::Add,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res},
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
use raid_race_lib::{
    AUTH_PORT, GAME_PORT, SERVER_ADDR,
    component::alive::{Cdr, status::Poison},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tracing::info;
use wtransport::tls::{Certificate, CertificateChain, PrivateKey};

#[derive(Resource, Deref, DerefMut, Default)]
pub struct Ids(Arc<RwLock<HashSet<u64>>>);

// VERSION 0
const PROTOCOL_ID: u64 = 0;
// Randomly generated
// TODO: env-based
const PRIVATE_KEY: Key = [
    184, 63, 110, 250, 164, 56, 107, 162, 244, 38, 79, 238, 202, 80, 29, 146, 241, 72, 217, 45,
    144, 145, 102, 85, 244, 9, 166, 80, 117, 193, 11, 0,
];

pub fn serve(mut commands: Commands, ids: Res<Ids>) {
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
        let (mut stream, _) = listener
            .accept()
            .await
            .expect("failed to accept auth connection");

        let id = loop {
            let out = rand::random();
            if !ids.read().await.contains(&out) {
                break out;
            }
        };

        let token = ConnectToken::build(
            // TODO:
            [
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), GAME_PORT),
                SocketAddr::new(SERVER_ADDR, GAME_PORT),
            ]
            .as_slice(),
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

        info!(id, "sent token to client");
    }
}

pub fn start_join(event: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(event.entity).insert(ReplicationSender);
}

pub fn join(
    event: On<Add, Connected>,
    id: Query<&RemoteId, With<ClientOf>>,
    ids: Res<Ids>,
    mut commands: Commands,
) {
    let Ok(RemoteId(id)) = id.get(event.entity) else {
        return;
    };

    entity::player(100, 40, 100)
        .build()
        .spawn(&mut commands, *id, event.entity);

    if let PeerId::Netcode(id) = id {
        ids.write_blocking().insert(*id);
    }
}

pub fn leave(event: On<Add, Disconnected>, id: Query<&RemoteId, With<ClientOf>>, ids: Res<Ids>) {
    if let Ok(RemoteId(PeerId::Netcode(id))) = id.get(event.entity) {
        ids.write_blocking().remove(id);
    };
}
