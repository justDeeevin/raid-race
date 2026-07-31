use crate::{component::alive::Agility, message::Buttons};
use avian3d::{dynamics::rigid_body::LinearVelocity, math::Vector};
use bevy::{time::Time, transform::components::Transform};

pub const JUMP_SPEED: f64 = 3.0;
pub const YAW_SENS: f32 = 0.003;

pub fn walk(
    input: Buttons,
    velocity: &mut LinearVelocity,
    transform: &Transform,
    agility: &Agility,
    time: &Time,
) {
    const WALK_SPEED: f64 = 5.0;
    const ACCELERATION_TIME_SECS: f64 = 0.1;
    const DECELERATION_TIME_SECS: f64 = ACCELERATION_TIME_SECS;

    let mut direction = Vector::ZERO;
    if input.contains(Buttons::FORWARD) {
        direction.z += 1.0
    }
    if input.contains(Buttons::BACKWARD) {
        direction.z -= 1.0
    }
    if input.contains(Buttons::LEFT) {
        direction.x += 1.0
    }
    if input.contains(Buttons::RIGHT) {
        direction.x -= 1.0
    }

    let movement_direction = transform
        .rotation
        .as_dquat()
        .mul_vec3(direction.normalize_or_zero());

    let target_velocity =
        movement_direction * (WALK_SPEED + (Agility::MOVE_SPEED_ADJUST * ***agility as f64));
    let blend_time = if movement_direction != Vector::ZERO {
        ACCELERATION_TIME_SECS
    } else {
        DECELERATION_TIME_SECS
    };

    let current_horizontal = Vector::new(velocity.x, 0.0, velocity.z);
    let target_horizontal = Vector::new(target_velocity.x, 0.0, target_velocity.z);

    let new_horizontal =
        current_horizontal.lerp(target_horizontal, time.delta_secs() as f64 / blend_time);

    velocity.x = new_horizontal.x;
    velocity.z = new_horizontal.z;
}
