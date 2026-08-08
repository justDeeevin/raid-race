use avian3d::{
    collision::{collider::Sensor, contact_types::Collisions},
    math::Vector,
};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    hierarchy::ChildOf,
    lifecycle::Add,
    observer::On,
    query::With,
    system::{Commands, Query},
};
use lightyear::prelude::input::bei::{Action, ActionOf, Complete, Start, TriggerState};
use raid_race_lib::{
    component::alive::player::{CanJump, Player},
    input::Jump,
};

#[derive(Component)]
pub struct Grounded;

pub fn grounded(
    mut commands: Commands,
    collisions: Collisions,
    sensors: Query<(Entity, &ChildOf), With<Sensor>>,
    grounded: Query<Entity, With<Grounded>>,
) {
    const MIN_GROUND_ANGLE: f64 = 30_f64.to_radians();

    for (sensor, ChildOf(parent)) in sensors {
        if collisions.collisions_with(sensor).any(|c| {
            c.manifolds
                .iter()
                .any(|m| m.normal.dot(Vector::Y).abs() >= MIN_GROUND_ANGLE.sin())
        }) {
            if grounded.get(*parent).is_err() {
                commands.entity(*parent).insert(Grounded);
            }
        } else if grounded.get(*parent).is_ok() {
            commands.entity(*parent).remove::<Grounded>();
        }
    }
}

pub fn landed(
    event: On<Add, Grounded>,
    mut commands: Commands,
    jumps: Query<&TriggerState, With<Action<Jump>>>,
) {
    if let Ok(jump) = jumps.get(event.entity)
        && *jump == TriggerState::None
    {
        commands.entity(event.entity).insert(CanJump);
    }
}

pub fn jump_released(
    event: On<Complete<Jump>>,
    mut commands: Commands,
    actions: Query<&ActionOf<Player>, With<Action<Jump>>>,
    grounded: Query<&Grounded>,
) {
    if let Ok(player) = actions.get(event.context)
        && grounded.get(**player).is_ok()
    {
        commands.entity(**player).insert(CanJump);
    }
}

pub fn leave_ground(
    event: On<Start<Jump>>,
    mut commands: Commands,
    actions: Query<&ActionOf<Player>, With<Action<Jump>>>,
) {
    if let Ok(player) = actions.get(event.context) {
        commands.entity(**player).remove::<CanJump>();
    }
}
