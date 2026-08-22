use axum::{Router, extract::Path, routing::get};
use bytes::Buf;
use futures::future::TryFutureExt;
use lightyear_netcode::ConnectToken;
use raid_race_lib::{AUTH_PORT, GAME_PORT, ID_PORT, PRIVATE_KEY, PROTOCOL_ID, SERVER_ADDR, TOTP};
use reqwest::Response;
use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/{server}", get(auth));
    let socket = TcpListener::bind(SocketAddr::new(SERVER_ADDR, AUTH_PORT))
        .await
        .expect("failed to bind to socket");
    axum::serve(socket, app).await.expect("failed to serve");
}

async fn auth(Path(server): Path<IpAddr>) -> [u8; 2048] {
    let id = reqwest::get(format!(
        "http://{server}:{ID_PORT}?totp={}",
        TOTP.generate_current()
    ))
    .and_then(Response::bytes)
    .await
    .expect("failed to get new id")
    .try_get_u64()
    .expect("id response didn't contain a u64");

    ConnectToken::build(
        SocketAddr::new(server, GAME_PORT),
        PROTOCOL_ID,
        id,
        PRIVATE_KEY,
    )
    .generate()
    .expect("failed to generate token")
    .try_into_bytes()
    .expect("failed to serialize token")
}
