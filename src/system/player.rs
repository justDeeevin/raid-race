use crate::{
    component::{
        OrbitCamera,
        alive::{Agility, player::PlayerMovable},
    },
    resource::Looking,
};
use avian3d::{
    collision::{
        collider::Sensor,
        collision_events::{CollisionEnd, CollisionStart},
    },
    dynamics::rigid_body::LinearVelocity,
    math::Vector,
};
use bevy::{
    ecs::{
        entity::Entity,
        hierarchy::ChildOf,
        observer::On,
        query::With,
        system::{Query, Res, ResMut},
    },
    input::{
        ButtonInput,
        keyboard::KeyCode,
        mouse::{AccumulatedMouseMotion, MouseButton},
    },
    math::{EulerRot, Quat},
    time::Time,
    transform::components::Transform,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub fn movement(
    query: Query<(
        &mut LinearVelocity,
        &Transform,
        Option<&Agility>,
        &mut PlayerMovable,
    )>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut velocity, transform, agility, mut state) in query {
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

            let movement_direction = transform
                .rotation
                .as_dquat()
                .mul_vec3(direction.normalize_or_zero());
            let target_velocity = movement_direction
                * (WALK_SPEED
                    + (Agility::MOVE_SPEED_ADJUST
                        * agility.map(|a| a.0).unwrap_or_default() as f64));
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
                        state.bhop = true;
                    }
                }
            }
        }
    }
}

pub fn land(
    collision: On<CollisionStart>,
    mut state: Query<&mut PlayerMovable>,
    sensor: Query<&ChildOf, With<Sensor>>,
) {
    let Ok(ChildOf(player)) = sensor.get(collision.collider1) else {
        return;
    };
    let mut state = state
        .get_mut(*player)
        .expect("Player not found for landing");

    if !state.airborne {
        tracing::warn!("Double landing");
    }
    state.airborne = false;
}

pub fn leave_ground(
    collision: On<CollisionEnd>,
    mut state: Query<&mut PlayerMovable>,
    sensor: Query<&ChildOf, With<Sensor>>,
) {
    let Ok(ChildOf(player)) = sensor.get(collision.collider1) else {
        return;
    };
    let mut state = state
        .get_mut(*player)
        .expect("Player not found for leaving ground");

    if state.airborne {
        tracing::warn!("Double leaving ground");
    }
    state.airborne = true;
}

const YAW_SENS: f32 = 0.003;

pub fn rotate(
    motion: Res<AccumulatedMouseMotion>,
    transform: Query<&mut Transform, With<PlayerMovable>>,
    looking: Res<Looking>,
) {
    if !**looking {
        return;
    }

    let delta = -motion.delta;
    let delta_yaw = delta.x * YAW_SENS;

    for mut transform in transform {
        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw + delta_yaw, pitch, roll);
    }
}

pub fn camera(
    mut transform: Query<&mut Transform>,
    camera: Query<(Entity, &OrbitCamera)>,
    motion: Res<AccumulatedMouseMotion>,
    looking: Res<Looking>,
) {
    const PITCH_SENS: f32 = YAW_SENS;
    const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

    let delta = -motion.delta;

    let delta_pitch = delta.y * PITCH_SENS;
    let delta_yaw = delta.x * YAW_SENS;

    for (camera, OrbitCamera { target, offset }) in camera {
        let target = transform
            .get(*target)
            .expect("Orbit camera target not found")
            .translation;

        let mut camera = transform.get_mut(camera).expect("Camera has no transform");

        if **looking {
            let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);
            camera.rotation = Quat::from_euler(
                EulerRot::YXZ,
                yaw + delta_yaw,
                (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT),
                roll,
            );
        }
        camera.translation =
            target - (camera.forward() * OrbitCamera::ORBIT_DISTANCE) + (camera.rotation * offset);
    }
}

pub fn grabber(
    click: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    mut looking: ResMut<Looking>,
    mut options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    #[allow(clippy::unwrap_used, reason = "statically safe")]
    let mut options = options.single_mut().unwrap();

    if **looking {
        if key.just_pressed(KeyCode::Escape) {
            **looking = false;
            options.grab_mode = CursorGrabMode::None;
            options.visible = true;
        }
    } else if click.just_pressed(MouseButton::Left) {
        **looking = true;
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }
}
