use bevy::{
    ecs::{
        component::{Component, Mutable},
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::Time,
};
use raid_race_lib::component::alive::{
    Dps, Health,
    status::{Poison, StackableStatusEffect},
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
            health.current = health.current.saturating_sub(poison.tick_damage(damagers));
        }
    }
}

pub fn stat_change<
    T: Component<Mutability = Mutable> + std::ops::DerefMut<Target = StackableStatusEffect>,
>(
    mut commands: Commands,
    time: Res<Time>,
    effects: Query<(Entity, &mut T)>,
) {
    for (entity, mut effect) in effects {
        effect.tick(time.delta());
        if effect.timer().is_finished() {
            commands.entity(entity).remove::<T>();
        }
    }
}
