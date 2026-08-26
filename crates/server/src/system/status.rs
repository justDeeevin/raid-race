use bevy::{ecs::component::Mutable, prelude::*};
use raid_race_lib::component::alive::{
    Dps, Health,
    status::{DefenseDown, DefenseUp, DpsDown, DpsUp, Poison, StackableStatusEffect},
};

fn poison(
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

fn stat_change<
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

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            poison,
            stat_change::<DefenseUp>,
            stat_change::<DefenseDown>,
            stat_change::<DpsUp>,
            stat_change::<DpsDown>,
        ),
    );
}
