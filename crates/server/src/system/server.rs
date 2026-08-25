use super::entity;
use crate::Ids;
use async_lock::RwLock;
use async_net::TcpListener;
use avian3d::parry::utils::hashset::HashSet;
use bevy::{
    app::{App, Startup},
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
use http_types::{Response, StatusCode};
use lightyear::{
    connection::{
        client::{Connected, Disconnected},
        client_of::ClientOf,
        server::Start,
    },
    core::id::{PeerId, RemoteId},
    link::server::LinkOf,
    netcode::{NetcodeServer, server_plugin::NetcodeConfig},
    prelude::{Identity, LocalAddr, ReplicationSender, server::ServerPlugins},
    webtransport::server::WebTransportServerIo,
};
use raid_race_lib::{GAME_PORT, ID_PORT, PRIVATE_KEY, PROTOCOL_ID, SERVER_ADDR, TICK_PERIOD, TOTP};
use std::{net::SocketAddr, sync::Arc};
use wtransport::tls::{Certificate, CertificateChain, PrivateKey};

#[derive(Resource, Deref, DerefMut, Default)]
struct ClientIds(Arc<RwLock<HashSet<u64>>>);

fn serve(mut commands: Commands, ids: Res<ClientIds>) {
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

    IoTaskPool::get().spawn(id_server(ids.clone())).detach();
}

async fn id_server(ids: Arc<RwLock<HashSet<u64>>>) {
    #[derive(serde::Deserialize)]
    struct Query {
        totp: String,
    }

    let socket = TcpListener::bind(SocketAddr::new(SERVER_ADDR, ID_PORT))
        .await
        .expect("failed to bind to socket");

    loop {
        let (stream, client_addr) = socket.accept().await.expect("failed to accept connection");
        let span = tracing::info_span!("id request", %client_addr);
        let _guard = span.enter();
        async_h1::accept(stream, async |req| {
            Ok(
                if let Ok(Query { totp }) = req.query()
                    && TOTP.check_current(&totp).is_some()
                {
                    let id: u64 = loop {
                        let out = rand::random();
                        if !ids.read().await.contains(&out) {
                            break out;
                        }
                    };
                    let mut res = Response::new(StatusCode::Ok);
                    res.set_body(id.to_be_bytes().as_slice());
                    tracing::info!(id, "sent new id");
                    res
                } else {
                    tracing::warn!("invalid totp");
                    Response::new(StatusCode::Unauthorized)
                },
            )
        })
        .await
        .expect("failed to send id");
    }
}

fn start_join(event: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(event.entity).insert(ReplicationSender);
}

fn join(
    event: On<Add, Connected>,
    id: Query<&RemoteId, With<ClientOf>>,
    client_ids: Res<ClientIds>,
    mut ids: ResMut<Ids>,
    mut commands: Commands,
) {
    let Ok(RemoteId(id)) = id.get(event.entity) else {
        return;
    };

    entity::player(100, 40, 10)
        .build()
        .spawn(&mut commands, *id, &mut ids, event.entity);

    if let PeerId::Netcode(id) = id {
        client_ids.write_blocking().insert(*id);
    }
}

fn leave(event: On<Add, Disconnected>, id: Query<&RemoteId, With<ClientOf>>, ids: Res<ClientIds>) {
    if let Ok(RemoteId(PeerId::Netcode(id))) = id.get(event.entity) {
        ids.write_blocking().remove(id);
    };
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        ServerPlugins {
            tick_duration: TICK_PERIOD,
        },
        raid_race_lib::plugin,
    ))
    .add_systems(Startup, serve)
    .add_observer(start_join)
    .add_observer(join)
    .add_observer(leave)
    .init_resource::<ClientIds>();
}
