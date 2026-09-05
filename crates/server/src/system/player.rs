use crate::cli::Args;
use bevy::prelude::*;
use lightyear::prelude::*;
use raid_race_lib::{
    Channel,
    component::alive::player::Player,
    event::{Attacked, NoCD},
    system::player::{PLAYER_CAPSULE_LENGTH, PLAYER_RADIUS},
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

fn spawn_visual(event: On<Add, Player>, args: Res<Args>, mut commands: Commands) {
    if args.visual {
        commands.entity(event.entity).apply_scene(bsn! {
            Mesh3d(asset_value(Capsule3d::new(PLAYER_RADIUS as f32, PLAYER_CAPSULE_LENGTH as f32)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
        });
    }
}

pub fn plugin(app: &mut App) {
    app.register_required_components_with::<NoCD, _>(|| Replicate::to_clients(NetworkTarget::All))
        .init_resource::<NoCD>()
        .add_observer(spawn_visual)
        .add_observer(alert_attack);
}
