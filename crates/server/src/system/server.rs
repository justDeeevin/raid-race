use crate::{
    component::Movement,
    resource::{Inputs, Room, UserEntities},
};
use avian3d::collision::collider::{Collider, Sensor};
use bevy::{
    ecs::{
        children,
        event::Event,
        message::MessageReader,
        system::{Commands, Res, ResMut},
    },
    transform::components::Transform,
};
use naia_bevy_server::{
    CommandsExt, Server, UserKey,
    events::{AuthEvents, ConnectEvent, DisconnectEvent, TickEvent},
    transport::webrtc,
};
use raid_race_lib::{
    channel,
    message::{self, Auth},
    system::entity::{self, PLAYER_HEIGHT, PLAYER_RADIUS},
};

#[derive(Event)]
pub struct Message<T: message::Trait> {
    pub message: T,
    pub user: UserKey,
}

#[derive(Event)]
pub struct Tick(pub u16);

pub fn serve(mut server: Server) {
    server.listen(webrtc::Socket::new(
        &webrtc::ServerAddrs::default(),
        server.socket_config(),
    ));
}

pub fn join(
    mut commands: Commands,
    mut server: Server,
    mut joins: MessageReader<ConnectEvent>,
    mut room: ResMut<Room>,
    mut user_entities: ResMut<UserEntities>,
) {
    let room = room.get_or_insert_with(|| server.create_room().key());

    for ConnectEvent(user) in joins.read() {
        const FOOT_HEIGHT: f64 = 0.02;

        let entity = entity::player(100, 40, 100)
            .build()
            .spawn(&mut commands)
            .insert((
                Movement::default(),
                children![(
                    Collider::cylinder(PLAYER_RADIUS, FOOT_HEIGHT),
                    Sensor,
                    Transform::from_xyz(0.0, ((-PLAYER_HEIGHT - FOOT_HEIGHT) / 2.0) as f32, 0.0),
                )],
            ))
            .enable_replication(&mut server)
            .id();
        server.room_mut(room).add_user(user).add_entity(&entity);
        user_entities.insert(*user, entity);
    }
}

pub fn auth(mut server: Server, mut events: MessageReader<AuthEvents>) {
    for events in events.read() {
        for (user_key, _) in events.read::<Auth>() {
            server.accept_connection(&user_key);
        }
    }
}

pub fn leave(
    mut commands: Commands,
    mut leaves: MessageReader<DisconnectEvent>,
    mut user_entities: ResMut<UserEntities>,
    mut inputs: ResMut<Inputs>,
    mut server: Server,
    room: Res<Room>,
) {
    for DisconnectEvent(user, _, _) in leaves.read() {
        if let Some(entity) = user_entities.remove(user) {
            // TODO: is this necessary?
            server
                .room_mut(room.0.as_ref().expect("no room"))
                .remove_user(user)
                .remove_entity(&entity);
            inputs.remove(&entity);
            commands.entity(entity).despawn();
        }
    }
}

pub fn tick(mut server: Server, mut ticks: MessageReader<TickEvent>, mut commands: Commands) {
    for TickEvent(tick) in ticks.read() {
        commands.trigger(Tick(*tick));
        let mut messages = server.receive_tick_buffer_messages(tick);

        macro_rules! trigger {
            ($($channel:ty: $message:ty),* $(,)?) => {
                $(for (user, message) in messages.read::<$channel, $message>() {
                    commands.trigger(Message { user, message });
                })*
            }
        }

        trigger!(channel::Input: message::Input);
    }
}
