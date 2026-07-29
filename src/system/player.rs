use crate::component::alive::{Agility, player::PlayerMovable};
use avian3d::{dynamics::rigid_body::LinearVelocity, math::Vector};
use bevy::{
    ecs::{
        query::With,
        system::{Query, Res},
    },
    input::{ButtonInput, keyboard::KeyCode},
    time::Time,
};

pub fn movement(
    query: Query<(&mut LinearVelocity, Option<&Agility>), With<PlayerMovable>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    const WALK_SPEED: f64 = 5.0;
    const ACCELERATION_RATE: f64 = 10.0;
    const DECELERATION_RATE: f64 = 8.0;

    for (mut velocity, agility) in query {
        let mut direction = Vector::ZERO;
        if input.pressed(KeyCode::KeyW) {
            direction.z += 1.0
        }
        if input.pressed(KeyCode::KeyS) {
            direction.z -= 1.0
        }
        if input.pressed(KeyCode::KeyA) {
            direction.x += 1.0
        }
        if input.pressed(KeyCode::KeyD) {
            direction.x -= 1.0
        }

        let movement_direction = direction.normalize_or_zero();
        let target_velocity = movement_direction
            * (WALK_SPEED + (0.1 * agility.map(|a| a.0).unwrap_or_default() as f64));
        let blend_speed = if movement_direction != Vector::ZERO {
            ACCELERATION_RATE
        } else {
            DECELERATION_RATE
        };

        let current_horizontal = Vector::new(velocity.x, 0.0, velocity.z);
        let target_horizontal = Vector::new(target_velocity.x, 0.0, target_velocity.z);

        let new_horizontal =
            current_horizontal.lerp(target_horizontal, blend_speed * time.delta_secs() as f64);

        velocity.x = new_horizontal.x;
        velocity.z = new_horizontal.z;
    }
}
