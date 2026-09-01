use crate::component::alive::{status::*, *};
use bevy::{ecs::component::Mutable, prelude::*};
use num_traits::SaturatingSub;
use std::ops::{AddAssign, DerefMut};

fn poison(
    mut commands: Commands,
    time: Res<Time>,
    afflicted: Query<(Entity, &mut Health, &mut Poison)>,
    damagers: Query<&Dps>,
) {
    for (entity, mut health, mut poison) in afflicted {
        poison.tick.tick(time.delta());

        if poison.total.tick(time.delta()).is_finished() {
            commands.entity(entity).remove::<Poison>();
        } else if poison.tick.just_finished() {
            health.current = health.current.saturating_sub(poison.tick_damage(damagers));
        }
    }
}

fn on_add<
    const DEBUFF: bool,
    T: Component + DerefMut<Target = StackableStatusEffect>,
    Target: Component<Mutability = Mutable> + DerefMut,
>(
    event: On<Add, T>,
    mut targets: Query<(&T, &mut Target)>,
) where
    Target::Target: Sized + From<u8> + AddAssign + SaturatingSub,
{
    if let Ok((effect, mut target)) = targets.get_mut(event.entity) {
        let delta = effect.stacks.get().into();
        if DEBUFF {
            **target = target.saturating_sub(&delta);
        } else {
            **target += delta;
        }
    }
}

fn on_remove<
    const DEBUFF: bool,
    T: Component + DerefMut<Target = StackableStatusEffect>,
    Target: Component<Mutability = Mutable> + DerefMut,
>(
    event: On<Remove, T>,
    mut targets: Query<(&T, &mut Target)>,
) where
    Target::Target: Sized + From<u8> + AddAssign + SaturatingSub,
{
    if let Ok((effect, mut target)) = targets.get_mut(event.entity) {
        let delta = effect.stacks.get().into();
        if DEBUFF {
            **target += delta;
        } else {
            **target = target.saturating_sub(&delta);
        }
    }
}

fn stat_change<T: Component<Mutability = Mutable> + DerefMut<Target = StackableStatusEffect>>(
    mut commands: Commands,
    time: Res<Time>,
    effects: Query<(Entity, &mut T)>,
) {
    for (entity, mut effect) in effects {
        if effect.timer.tick(time.delta()).is_finished() {
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
            stat_change::<AgilityUp>,
            stat_change::<AgilityDown>,
        ),
    )
    // TODO: maybe make this less ugly
    .add_observer(on_add::<false, DefenseUp, Defense>)
    .add_observer(on_remove::<false, DefenseUp, Defense>)
    .add_observer(on_add::<true, DefenseDown, Defense>)
    .add_observer(on_remove::<true, DefenseDown, Defense>)
    .add_observer(on_add::<false, DpsUp, Dps>)
    .add_observer(on_remove::<false, DpsUp, Dps>)
    .add_observer(on_add::<true, DpsDown, Dps>)
    .add_observer(on_remove::<true, DpsDown, Dps>)
    .add_observer(on_add::<false, AgilityUp, Agility>)
    .add_observer(on_remove::<false, AgilityUp, Agility>)
    .add_observer(on_add::<true, AgilityDown, Agility>)
    .add_observer(on_remove::<true, AgilityDown, Agility>);
}
