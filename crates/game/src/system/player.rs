use crate::component::OrbitCamera;
use avian3d::physics_transform::{Position, Rotation};
use bevy::{
    asset::asset_value,
    camera::Camera3d,
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        hierarchy::Children,
        lifecycle::{Add, Insert},
        observer::On,
        query::{With, Without},
        spawn::SpawnRelated,
        system::{Commands, Query},
    },
    input::keyboard::KeyCode,
    math::{
        EulerRot, Quat, Vec3,
        primitives::{Capsule3d, Cuboid},
    },
    mesh::Mesh3d,
    pbr::{MeshMaterial3d, StandardMaterial},
    scene::{EntityCommandsSceneExt, bsn},
    transform::components::Transform,
};
use bevy_console::{
    ConsoleCommand,
    clap::{self, Parser},
};
use lightyear::prelude::{
    Controlled,
    input::bei::{Action, ActionOf, Binding, Bindings, Cardinal, InputAction, bindings},
};
use raid_race_lib::{
    component::alive::{
        Id,
        player::{Pitch, Player},
    },
    input::{Jump, Look, Walk},
    player::{PLAYER_CAPSULE_LENGTH, PLAYER_RADIUS, physics_components},
};

#[derive(Parser, ConsoleCommand)]
#[command(name = "whoami")]
/// Print the entity ID of the currently controlled player
pub struct WhoAmI;

pub fn whoami(
    mut command: ConsoleCommand<WhoAmI>,
    player: Query<&Id, (With<Player>, With<Controlled>)>,
) {
    if command.take().is_none_or(|c| c.is_err()) {
        return;
    }

    if let Ok(id) = player.single() {
        command.reply_ok(id.to_string());
    } else {
        command.reply_failed("no player found");
    }
}

// TODO: configurable
pub trait Binds: InputAction {
    fn bindings() -> impl Bundle;
}

impl Binds for Walk {
    fn bindings() -> impl Bundle {
        Bindings::spawn(Cardinal::wasd_keys())
    }
}

impl Binds for Jump {
    fn bindings() -> impl Bundle {
        bindings![KeyCode::Space]
    }
}

impl Binds for Look {
    fn bindings() -> impl Bundle {
        bindings![Binding::mouse_motion()]
    }
}

/// Adds bindings to an action entity when it spawns
pub fn add_bindings_on_action_spawn<A: Binds, Context: Component, Owner: Component>(
    event: On<Insert, (Action<A>, ActionOf<Context>)>,
    actions: Query<&ActionOf<Context>, (With<Action<A>>, Without<Bindings>)>,
    controlled: Query<(), (With<Owner>, With<Controlled>)>,
    mut commands: Commands,
) {
    if let Ok(action_of) = actions.get(event.entity)
        && controlled.get(**action_of).is_ok()
    {
        commands.entity(event.entity).insert(A::bindings());
    }
}

/// Generates an observer system closure that adds bindings to actions owned by entities with the
/// given component[^1] when the owner spawns.
///
/// This is for the case where the actions spawn before the owner. Actions that spawn after the
/// owner will be handled by [`add_bindings_on_action_spawn`].
///
/// [^1]: That is, their `ActionOf` will target those entities.
macro_rules! add_bindings_on_owner_spawn {
    ($owner:ty {$($owners:ident: $context:ty[$($actions:ident: $action:ty),* $(,)?]),* $(,)?}) => {{
        use ::lightyear::{input::bei::{self, prelude::Actions}, prelude::Controlled};
        use ::bevy::ecs::{self, system::{self, Query}, query::{self, With}};

        |
            event: ecs::observer::On<ecs::lifecycle::Add, ($owner, Controlled, $(Actions<$context>),*)>,
            $($owners: Query<&Actions<$context>, (With<$owner>, With<Controlled>)>,
                $($actions: Query<(), (With<bei::prelude::Action<$action>>, query::Without<bei::prelude::Bindings>)>),*
            ),*,
            mut commands: system::Commands,
        | {$(
            if let Ok(actions) = $owners.get(event.entity)  {
                for action in actions {$(
                    if $actions.get(action).is_ok() {
                      commands.entity(action).insert(<$action as $crate::system::player::Binds>::bindings());
                      continue;
                    }
                )*}
            }
        )*}
    }}
}

pub(crate) use add_bindings_on_owner_spawn;

pub fn spawn(
    event: On<Add, (Player, Controlled)>,
    players: Query<(), (With<Player>, With<Controlled>)>,
    mut commands: Commands,
    camera: Query<Entity, With<Camera3d>>,
) {
    const CAMERA_OFFSET: Vec3 = Vec3::new(1.0, 1.0, 0.0);

    if players.get(event.entity).is_err() {
        return;
    }

    commands
        .entity(event.entity)
        .apply_scene(bsn!(
            #Player
            Mesh3d(asset_value(Capsule3d::new(
                PLAYER_RADIUS as f32,
                PLAYER_CAPSULE_LENGTH as f32,
            )))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            Children [
                Mesh3d(asset_value(Cuboid::new(0.1, 0.1, 0.5)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
                Transform::from_xyz(0.0, 0.6, -0.5)
            ]
        ))
        .insert(physics_components());

    commands
        .entity(camera.single().expect("multiple cameras"))
        .insert(OrbitCamera {
            target: event.entity,
            offset: CAMERA_OFFSET,
        });
}

pub fn orbit(
    targets: Query<(&Position, &Rotation, &Pitch)>,
    cameras: Query<(&mut Transform, &OrbitCamera)>,
) {
    for (mut transform, OrbitCamera { target, offset }) in cameras {
        let (position, rotation, pitch) =
            targets.get(*target).expect("orbit camera target not found");

        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            **pitch,
            rotation.to_euler(EulerRot::XYZ).1 as f32,
            transform.rotation.to_euler(EulerRot::XYZ).2,
        );

        transform.translation = position.as_vec3()
            - (transform.forward() * OrbitCamera::ORBIT_DISTANCE)
            + (transform.rotation * offset);
    }
}
