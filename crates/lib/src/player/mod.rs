pub mod character;
pub mod weapon;

use crate::{
    component::alive::{
        Agility, Dps, Health,
        player::{AttackTimer, Grounded, Pitch, Player},
    },
    event::{Attacked, Hit},
    input::{Attack, Jump, Look, Walk},
    player::character::Cooldowns,
    scene::Dummy,
};
use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{
        LinearVelocity, LockedAxes, RigidBody,
        forces::{Forces, ReadRigidBodyForces, WriteRigidBodyForces},
        mass_properties::components::ComputedMass,
    },
    math::Vector,
    physics_transform::{Position, Rotation},
    spatial_query::{RayCaster, RayHits},
};
use bevy::{
    app::{App, FixedUpdate},
    ecs::{
        bundle::Bundle,
        entity::Entity,
        observer::On,
        query::With,
        system::{Commands, Query, Res},
    },
    math::{DQuat, Dir3, EulerRot, Quat, Vec3},
    time::Time,
    transform::components::Transform,
};
use bevy_enhanced_input::action::events::{Fire, Start};
use either::Either;

pub const PLAYER_RADIUS: f64 = 0.5;
pub const PLAYER_HEIGHT: f64 = 2.0;
// -- DON'T CHANGE --
pub const PLAYER_CAPSULE_LENGTH: f64 = PLAYER_HEIGHT - (PLAYER_RADIUS * 2.0);
// -------------------

fn walk(
    event: On<Fire<Walk>>,
    mut params: Query<(&ComputedMass, &Transform, &Agility, Forces)>,
    time: Res<Time>,
) {
    const MAX_SPEED: f32 = 5.0;
    const MAX_ACCELERATION: f64 = 40.0;

    let Ok((mass, transform, agility, mut forces)) = params.get_mut(event.context) else {
        return;
    };

    let delta_t = time.delta_secs_f64();

    let max_delta_v = MAX_ACCELERATION * delta_t;

    let Ok(move_dir) =
        Dir3::new(Vec3::new(event.value.x, 0.0, -event.value.y)).map(|d| transform.rotation * d)
    else {
        return;
    };

    let velocity = {
        let t = forces.linear_velocity();
        Vector::new(t.x, 0.0, t.z)
    };

    let target_velocity = move_dir
        .map(|d| d * (MAX_SPEED + (Agility::MOVE_SPEED_ADJUST * **agility as f32)))
        .as_dvec3();

    let new_velocity = velocity.move_towards(target_velocity, max_delta_v);

    let required_acceleration = (new_velocity - velocity) / delta_t;

    forces.apply_force(required_acceleration * mass.value());
}

fn jump(event: On<Start<Jump>>, mut velocity: Query<&mut LinearVelocity, With<Grounded>>) {
    const JUMP_SPEED: f64 = 3.0;

    if let Ok(mut velocity) = velocity.get_mut(event.context) {
        velocity.y = JUMP_SPEED
    }
}

fn look(event: On<Fire<Look>>, mut params: Query<(&mut Pitch, &mut Rotation)>) {
    const YAW_SENS: f64 = 0.003;
    const PITCH_SENS: f64 = YAW_SENS;
    const MAX_PITCH: f64 = std::f64::consts::FRAC_PI_2;

    let delta = -event.value;

    let Ok((mut pitch, mut rotation)) = params.get_mut(event.context) else {
        return;
    };

    **pitch = (**pitch + (delta.y as f64 * PITCH_SENS)).clamp(-MAX_PITCH, MAX_PITCH);

    let (yaw, pitch, roll) = rotation.to_euler(EulerRot::YXZ);
    **rotation = DQuat::from_euler(
        EulerRot::YXZ,
        yaw + (delta.x as f64 * YAW_SENS),
        pitch,
        roll,
    );
}

pub fn camera_transform(target: (&Position, &Rotation, &Pitch)) -> Transform {
    const CAMERA_OFFSET: Vec3 = Vec3::new(1.0, 1.0, 0.0);
    const CAMERA_DISTANCE: f32 = 5.0;

    let (position, rotation, pitch) = target;
    let mut out = Transform::default();

    out.rotation = Quat::from_euler(
        EulerRot::YXZ,
        rotation.to_euler(EulerRot::YXZ).0 as f32,
        **pitch as f32,
        out.rotation.to_euler(EulerRot::YXZ).2,
    );
    out.translation =
        position.as_vec3() - (out.forward() * CAMERA_DISTANCE) + (out.rotation * CAMERA_OFFSET);

    out
}

fn grounded(
    casts: Query<(Entity, &RayHits), With<Player>>,
    grounded: Query<(), With<Grounded>>,
    mut commands: Commands,
) {
    const MIN_ANGLE: f64 = 30_f64.to_radians();
    const MAX_DISTANCE: f64 = 0.1;

    let sin = MIN_ANGLE.sin();

    for (player, hits) in casts {
        if hits.iter().any(|hit| {
            hit.normal.dot(Vector::Y) >= hit.normal.length() * sin && hit.distance <= MAX_DISTANCE
        }) {
            if grounded.get(player).is_err() {
                commands.entity(player).insert(Grounded);
            }
        } else if grounded.get(player).is_ok() {
            commands.entity(player).remove::<Grounded>();
        }
    }
}

fn attack(
    event: On<Fire<Attack>>,
    mut attack_timer: Query<&mut AttackTimer>,
    mut commands: Commands,
) {
    if let Ok(mut attack_timer) = attack_timer.get_mut(event.context)
        && attack_timer.is_finished()
    {
        attack_timer.reset();
        commands.trigger(Attacked(event.context));
    }
}

fn dummy(health: Query<&mut Health, With<Dummy>>) {
    for mut health in health {
        if health.current < health.cap {
            tracing::info!(damage = health.cap - health.current, "dummy hit");
        }
        health.current = health.cap;
    }
}

fn ability_cooldown(cooldowns: Query<&mut Cooldowns>, time: Res<Time>) {
    for mut cooldowns in cooldowns {
        for cooldown in &mut **cooldowns {
            if let Either::Left(timer) = cooldown {
                timer.tick(time.delta());
            }
        }
    }
}

fn hit(event: On<Hit>, damage: Query<(&Dps, &AttackTimer)>, mut health: Query<&mut Health>) {
    let Ok((Dps(damage), AttackTimer(timer))) = damage.get(event.source) else {
        return;
    };

    if let Ok(mut health) = health.get_mut(event.target) {
        health.current = health
            .current
            .saturating_sub((*damage as f32 * timer.duration().as_secs_f32()) as u16);
    }
}

fn attack_cooldown(attack_timer: Query<&mut AttackTimer>, time: Res<Time>) {
    for mut timer in attack_timer {
        timer.tick(time.delta());
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (grounded, dummy, ability_cooldown, attack_cooldown),
    )
    .add_observer(walk)
    .add_observer(jump)
    .add_observer(look)
    .add_observer(attack)
    .add_observer(hit)
    .add_observer(weapon::placeholder_gun::shoot)
    .add_plugins(character::warrior::plugin);
}

pub fn physics_components() -> impl Bundle {
    (
        RigidBody::Dynamic,
        Collider::capsule(PLAYER_RADIUS, PLAYER_CAPSULE_LENGTH),
        LockedAxes::ROTATION_LOCKED,
        RayCaster::new(Vector::new(0.0, -PLAYER_HEIGHT / 2.0, 0.0), Dir3::NEG_Y),
    )
}
