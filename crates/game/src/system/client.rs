use bevy::ecs::{event::Event, message::MessageReader, observer::On, system::Commands};
use naia_bevy_client::{events::ClientTickEvent, transport::webrtc, Client, DefaultClientTag};
use raid_race_lib::message::Auth;

use crate::Quit;

#[derive(Event)]
pub struct Tick(pub naia_bevy_client::Tick);

pub fn connect(mut client: Client<DefaultClientTag>) {
    const URL: &str = "http://127.0.0.1:14191";

    client.auth(Auth);
    client.connect(webrtc::Socket::new(URL, client.socket_config()));
}

pub fn tick(mut ticks: MessageReader<ClientTickEvent<DefaultClientTag>>, mut commands: Commands) {
    for event in ticks.read() {
        commands.trigger(Tick(event.tick))
    }
}

pub fn disconnect(_: On<Quit>, mut client: Client<DefaultClientTag>) {
    client.disconnect();
}
