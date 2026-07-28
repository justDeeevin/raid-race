use crate::component::alive::{Dps, Health, status::Poison};
use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::Time,
};

pub fn poison(
    mut commands: Commands,
    time: Res<Time>,
    afflicted: Query<(Entity, &mut Health, &mut Poison)>,
    damagers: Query<&Dps>,
) {
    for (entity, mut health, mut poison) in afflicted {
        poison.tick.tick(time.delta());
        poison.total.tick(time.delta());
        if poison.total.is_finished() {
            commands.entity(entity).remove::<Poison>();
        } else if poison.tick.just_finished() {
            health.0 = health.0.saturating_sub(poison.tick_damage(damagers));
        }
    }
}
