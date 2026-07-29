use crate::component::alive::{Agility, player::PlayerMovable};
use avian3d::{dynamics::rigid_body::LinearVelocity, math::Vector};
use bevy::{
    ecs::system::{Query, Res},
    input::{ButtonInput, keyboard::KeyCode},
    time::Time,
};

pub fn movement(
    query: Query<(&mut LinearVelocity, Option<&Agility>, &mut PlayerMovable)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut velocity, agility, mut state) in query {
        // Walking
        {
            const WALK_SPEED: f64 = 5.0;
            const ACCELERATION_TIME: f64 = 0.1;
            const DECELERATION_TIME: f64 = ACCELERATION_TIME;

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
            let blend_time = if movement_direction != Vector::ZERO {
                ACCELERATION_TIME
            } else {
                DECELERATION_TIME
            };

            let current_horizontal = Vector::new(velocity.x, 0.0, velocity.z);
            let target_horizontal = Vector::new(target_velocity.x, 0.0, target_velocity.z);

            let new_horizontal =
                current_horizontal.lerp(target_horizontal, time.delta_secs() as f64 / blend_time);

            velocity.x = new_horizontal.x;
            velocity.z = new_horizontal.z;
        }

        // Jumping
        {
            const JUMP_SPEED: f64 = 3.0;

            if !state.airborne {
                if state.bhop {
                    if !input.pressed(KeyCode::Space) {
                        state.bhop = false;
                    }
                } else {
                    if input.just_pressed(KeyCode::Space) {
                        velocity.y = JUMP_SPEED;
                        state.airborne = true;
                        state.bhop = true;
                    }
                }
            } else if velocity.y < 0.0 {
                state.airborne = false;
            }
        }
    }
}
