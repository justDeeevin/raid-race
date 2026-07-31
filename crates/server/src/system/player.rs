use crate::{
    component::Movement,
    resource::{Inputs, UserEntities},
    system::server::{Message, Tick},
};
use avian3d::{
    collision::{collider::Sensor, contact_types::Collisions},
    dynamics::rigid_body::LinearVelocity,
    math::Vector,
};
use bevy::{
    ecs::{
        entity::Entity,
        hierarchy::ChildOf,
        observer::On,
        query::{Changed, Or, With},
        system::{Query, Res, ResMut},
    },
    time::Time,
    transform::components::Transform,
};
use raid_race_lib::{
    component::alive::{Agility, player::SimSync},
    message::{Buttons, Input},
    system::player::{JUMP_SPEED, walk},
};

pub fn receive_input(
    event: On<Message<Input>>,
    user_entities: Res<UserEntities>,
    mut inputs: ResMut<Inputs>,
) {
    #[allow(
        clippy::unwrap_used,
        reason = "user_entities stays up-to-date with connected users; if a user sends a message, it's guarenteed to be in the table"
    )]
    let entity = *user_entities.get(&event.user).unwrap();

    tracing::debug!(?event.user, ?event.message, "input");

    inputs.entry(entity).or_default().apply(event.message);
}

pub fn apply_input(
    mut params: Query<(&mut LinearVelocity, &mut Transform, &Agility, &mut Movement)>,
    time: Res<Time>,
    inputs: Res<Inputs>,
) {
    for (entity, input) in &inputs.0 {
        let (mut velocity, transform, agility, mut state) = params
            .get_mut(*entity)
            .expect("user entity is missing necessary components for movement");
        walk(*input, &mut velocity, &transform, agility, &time);

        if state.ground_contacts != 0 {
            let jump = input.contains(Buttons::JUMP);
            if state.bhop {
                if !jump {
                    state.bhop = false;
                }
            } else if jump {
                velocity.y = JUMP_SPEED;
                state.bhop = true;
            }
        }
    }
}

pub fn grounded(
    collisions: Collisions,
    mut state: Query<&mut Movement>,
    sensors: Query<(Entity, &ChildOf), With<Sensor>>,
) {
    const MIN_GROUND_ANGLE: f64 = 30_f64.to_radians();

    for (sensor, ChildOf(parent)) in sensors {
        let Ok(mut state) = state.get_mut(*parent) else {
            continue;
        };
        state.ground_contacts = 0;
        if collisions.collisions_with(sensor).any(|c| {
            c.manifolds
                .iter()
                .any(|m| m.normal.dot(Vector::Y).abs() >= MIN_GROUND_ANGLE.sin())
        }) {
            state.ground_contacts += 1;
        }
    }
}

#[allow(clippy::type_complexity, reason = "fuq off")]
pub fn sync_sim(
    _: On<Tick>,
    simulated: Query<
        (Entity, &Transform, &LinearVelocity),
        Or<(Changed<Transform>, Changed<LinearVelocity>)>,
    >,
    mut replicated: Query<&mut SimSync>,
) {
    for (entity, transform, velocity) in simulated {
        let Ok(mut sim) = replicated.get_mut(entity) else {
            continue;
        };
        *sim.translation = transform.translation.into();
        *sim.velocity = velocity.0.into();
    }
}
