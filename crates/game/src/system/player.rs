use super::client::Tick;
use crate::{
    component::OrbitCamera,
    resource::{InputState, Inputs, Looking, Me},
};
use avian3d::{dynamics::rigid_body::LinearVelocity, parry::utils::hashmap::HashMap};
use bevy::{
    asset::Assets,
    camera::Camera3d,
    color::Color,
    ecs::{
        change_detection::DetectChanges,
        children,
        entity::Entity,
        message::MessageReader,
        observer::On,
        query::{Changed, With},
        system::{Commands, Query, Res, ResMut},
    },
    input::{
        ButtonState,
        keyboard::{KeyCode, KeyboardInput},
        mouse::{AccumulatedMouseMotion, MouseButton, MouseButtonInput},
    },
    math::{
        EulerRot, Quat, Vec3,
        primitives::{Capsule3d, Cuboid},
    },
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    time::Time,
    transform::components::Transform,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_console::ConsoleOpen;
use naia_bevy_client::{
    Client, DefaultClientTag,
    events::{DespawnEntityEvent, SpawnEntityEvent},
};
use raid_race_lib::{
    channel,
    component::alive::{Agility, player::SimSync},
    message::{Button, Input},
    system::{
        entity::{self, PLAYER_CAPSULE_LENGTH, PLAYER_RADIUS},
        player::{YAW_SENS, walk},
    },
};

pub fn spawn(
    mut spawned: MessageReader<SpawnEntityEvent<DefaultClientTag>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<Entity, With<Camera3d>>,
    me: Option<Res<Me>>,
) {
    if let Some(event) = spawned.read().next() {
        #[warn(clippy::unwrap_used, reason = "tmp")]
        let camera = cameras.single().unwrap();
        let player = event.entity;

        const CAMERA_OFFSET: Vec3 = Vec3::new(1.0, 1.0, 0.0);
        commands.entity(event.entity).insert((
            entity::player(100, 40, 100).build().bundle(),
            Mesh3d(meshes.add(Capsule3d::new(
                PLAYER_RADIUS as f32,
                PLAYER_CAPSULE_LENGTH as f32,
            ))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            children![(
                Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.5))),
                MeshMaterial3d(materials.add(Color::WHITE)),
                Transform::from_xyz(0.0, 0.6, 0.5),
            )],
        ));

        commands.entity(camera).insert(OrbitCamera {
            target: player,
            offset: CAMERA_OFFSET,
        });

        if me.is_none() {
            commands.insert_resource(Me(player));
            super::ui::hud::spawn(commands.reborrow(), player);
        }
    }
}

pub fn despawn(
    mut commands: Commands,
    mut despawns: MessageReader<DespawnEntityEvent<DefaultClientTag>>,
    cameras: Query<(Entity, &OrbitCamera)>,
    me: Option<Res<Me>>,
) {
    let cameras = cameras
        .iter()
        .map(|(e, camera)| (camera.target, e))
        .collect::<HashMap<_, _>>();

    for event in despawns.read() {
        if let Some(camera) = cameras.get(&event.entity).copied() {
            commands.entity(camera).despawn();
        }
        if let Some(me) = &me
            && ***me == event.entity
        {
            commands.remove_resource::<Me>();
        }
    }
}

pub fn read_input(
    mut buttons: MessageReader<KeyboardInput>,
    mut inputs: ResMut<Inputs>,
    mut input_state: ResMut<InputState>,
    console_state: Res<ConsoleOpen>,
) {
    for KeyboardInput {
        key_code,
        state,
        repeat,
        ..
    } in buttons.read()
    {
        if *repeat {
            continue;
        }

        // TODO: this should eventually be configurable
        let button = match key_code {
            KeyCode::KeyW => Button::Forward,
            KeyCode::KeyD => Button::Right,
            KeyCode::KeyS => Button::Backward,
            KeyCode::KeyA => Button::Left,
            KeyCode::Space => Button::Jump,
            _ => continue,
        };

        let input = match state {
            ButtonState::Pressed => {
                if console_state.open {
                    continue;
                } else {
                    Input::Pressed(button)
                }
            }
            ButtonState::Released => Input::Released(button),
        };

        input_state.apply(input);
        inputs.push(input);
    }
}

pub fn simulate_input(
    input: Res<InputState>,
    mut params: Query<(&mut LinearVelocity, &Transform, &Agility)>,
    time: Res<Time>,
    me: Res<Me>,
) {
    let (mut velocity, transform, agility) = params
        .get_mut(**me)
        .expect("player is missing necessary components for movement");

    walk(**input, &mut velocity, transform, agility, &time);
}

pub fn send_input(
    tick: On<Tick>,
    mut client: Client<DefaultClientTag>,
    mut inputs: ResMut<Inputs>,
) {
    for input in inputs.drain(..) {
        tracing::info!(?input, "sending input");
        client.send_tick_buffer_message::<channel::Input, _>(&tick.0, &input);
    }
}

pub fn sync_sim(query: Query<(&SimSync, &mut Transform, &mut LinearVelocity), Changed<SimSync>>) {
    for (sim, mut transform, mut velocity) in query {
        transform.translation = (*sim.translation).into();
        **velocity = (*sim.velocity).into();
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

        if **looking && motion.is_changed() {
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
    mut clicks: MessageReader<MouseButtonInput>,
    mut key: MessageReader<KeyboardInput>,
    mut looking: ResMut<Looking>,
    mut options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    console_state: Res<ConsoleOpen>,
) {
    #[allow(clippy::unwrap_used, reason = "there's only one primary window")]
    let mut options = options.single_mut().unwrap();

    let click = clicks.read().any(|e| e.button == MouseButton::Left);
    let esc = key.read().any(|e| e.key_code == KeyCode::Escape);

    if esc && click {
    } else if **looking {
        if esc {
            **looking = false;
            options.grab_mode = CursorGrabMode::None;
            options.visible = true;
        }
    } else if click && !console_state.open {
        **looking = true;
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }
}
