use bevy::{
    app::App,
    ecs::{
        observer::On,
        system::{Query, Single},
    },
};
use lightyear::{
    connection::network_target::Target,
    core::id::RemoteId,
    link::server::Server,
    prelude::{ControlledBy, ServerMultiMessageSender},
};
use raid_race_lib::{Channel, event::Attacked};

fn alert_attack(
    event: On<Attacked>,
    mut tx: ServerMultiMessageSender,
    server: Single<&Server>,
    controlled: Query<&ControlledBy>,
    ids: Query<&RemoteId>,
) {
    let target = if let Ok(client) = controlled.get(**event)
        && let Ok(RemoteId(id)) = ids.get(client.owner)
    {
        Target::AllExceptSingle(*id)
    } else {
        Target::All
    };

    if let Err(e) = tx.send::<Attacked, Channel>(&event, &server, &target) {
        tracing::error!(%e, "Failed to send attacked message");
    }
}

pub fn plugin(app: &mut App) {
    app.add_observer(alert_attack);
}
