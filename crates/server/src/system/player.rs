use bevy::prelude::*;
use lightyear::{connection::network_target::Target, prelude::*};
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

    #[allow(
        clippy::unwrap_used,
        reason = "should never fail because channel is reliable"
    )]
    tx.send::<Attacked, Channel>(&event, &server, &target)
        .unwrap();
}

pub fn plugin(app: &mut App) {
    app.add_observer(alert_attack);
}
