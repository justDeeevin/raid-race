use crate::{
    component::alive::{Agility, player::Pitch},
    input::{Jump, Look, Walk},
};
use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{
        Friction, LinearVelocity, LockedAxes, RigidBody,
        forces::{Forces, ReadRigidBodyForces, WriteRigidBodyForces},
        mass_properties::components::ComputedMass,
    },
    math::Vector,
};
use bevy::{
    app::App,
    ecs::{
        bundle::Bundle,
        observer::On,
        system::{Query, Res},
    },
    math::{Dir3, EulerRot, Quat, Vec3},
    time::Time,
    transform::components::Transform,
};
use bevy_enhanced_input::action::events::{Fire, Start};

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
    const MAX_ACCELERATION: f64 = 20.0;

    let Ok((mass, transform, agility, mut forces)) = params.get_mut(event.context) else {
        return;
    };

    let delta_t = time.delta_secs_f64();

    let max_delta_v = MAX_ACCELERATION * delta_t;

    let move_dir = Dir3::new(Vec3::new(event.value.x, 0.0, -event.value.y))
        .ok()
        .map(|d| transform.rotation * d);

    let velocity = {
        let t = forces.linear_velocity();
        Vector::new(t.x, 0.0, t.z)
    };

    let target_velocity = move_dir
        .map(|d| d * (MAX_SPEED + (Agility::MOVE_SPEED_ADJUST * **agility as f32)))
        .unwrap_or_default()
        .as_dvec3();

    let new_velocity = velocity.move_towards(target_velocity, max_delta_v);

    let required_acceleration = (new_velocity - velocity) / delta_t;

    forces.apply_force(required_acceleration * mass.value());
}

fn jump(event: On<Start<Jump>>, mut velocity: Query<&mut LinearVelocity>) {
    const JUMP_SPEED: f64 = 3.0;

    if let Ok(mut velocity) = velocity.get_mut(event.context) {
        velocity.y = JUMP_SPEED
    }
}

fn look(event: On<Fire<Look>>, mut params: Query<(&mut Pitch, &mut Transform)>) {
    const YAW_SENS: f32 = 0.003;
    const PITCH_SENS: f32 = YAW_SENS;

    let Ok((mut pitch, mut transform)) = params.get_mut(event.context) else {
        return;
    };

    **pitch += event.value.x * PITCH_SENS;

    let (roll, old_yaw, pitch) = transform.rotation.to_euler(EulerRot::XYZ);
    transform.rotation = Quat::from_euler(
        EulerRot::XYZ,
        roll,
        old_yaw + (event.value.y * YAW_SENS),
        pitch,
    );
}

pub fn plugin(app: &mut App) {
    app.add_observer(walk).add_observer(jump).add_observer(look);
}

pub fn physics_components() -> impl Bundle {
    (
        RigidBody::Dynamic,
        Collider::capsule(PLAYER_RADIUS, PLAYER_CAPSULE_LENGTH),
        LockedAxes::ROTATION_LOCKED,
        Friction::new(0.0),
    )
}
