use bevy::prelude::*;
use lightyear::{connection::network_target::Target, prelude::*};
use raid_race_lib::{
    Channel,
    event::{Attacked, NoCD},
};

fn alert_attack(
    event: On<Attacked>,
    sources: Query<&ControlledBy>,
    mut senders: Query<(Entity, &mut EventSender<Attacked>)>,
) {
    let Ok(source) = sources.get(**event) else {
        return;
    };

    for mut sender in senders.iter_mut().filter_map(|(entity, sender)| {
        if entity == source.owner {
            None
        } else {
            Some(sender)
        }
    }) {
        sender.trigger::<Channel>(event.clone());
    }
}

pub fn plugin(app: &mut App) {
    app.register_required_components_with::<NoCD, _>(|| Replicate::to_clients(Target::All))
        .init_resource::<NoCD>()
        .add_observer(alert_attack);
}
