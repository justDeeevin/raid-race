pub mod weapon;

use crate::component::{AimCamera, OrbitCamera};
use avian3d::physics_transform::{Position, Rotation};
use bevy::{
    asset::asset_value,
    audio::SpatialListener,
    color::Color,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        hierarchy::Children,
        lifecycle::{Add, Insert},
        message::MessageReader,
        observer::On,
        query::{With, Without},
        spawn::SpawnRelated,
        system::{Commands, Query, Single},
    },
    input::{
        ButtonState,
        keyboard::{KeyCode, KeyboardInput},
        mouse::{MouseButton, MouseButtonInput},
    },
    math::primitives::{Capsule3d, Cuboid},
    mesh::Mesh3d,
    pbr::{MeshMaterial3d, StandardMaterial},
    scene::{EntityCommandsSceneExt, bsn},
    transform::components::Transform,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_console::{
    ConsoleCommand,
    clap::{self, Parser},
};
use lightyear::prelude::{
    Controlled,
    input::bei::{
        Action, ActionOf, Binding, Bindings, Cardinal, ContextActivity, InputAction, bindings,
    },
};
use raid_race_lib::{
    component::alive::{
        Id,
        player::{Pitch, Player},
    },
    input::{Ability, Attack, Jump, Look, Walk},
    player::{PLAYER_CAPSULE_LENGTH, PLAYER_RADIUS, camera_transform, physics_components},
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

impl Binds for Ability<1> {
    fn bindings() -> impl Bundle {
        bindings![KeyCode::Digit1]
    }
}

impl Binds for Ability<2> {
    fn bindings() -> impl Bundle {
        bindings![KeyCode::Digit2]
    }
}

impl Binds for Ability<3> {
    fn bindings() -> impl Bundle {
        bindings![KeyCode::Digit3]
    }
}

impl Binds for Ability<4> {
    fn bindings() -> impl Bundle {
        bindings![KeyCode::Digit4]
    }
}

impl Binds for Ability<5> {
    fn bindings() -> impl Bundle {
        bindings![KeyCode::Digit5]
    }
}

impl Binds for Attack {
    fn bindings() -> impl Bundle {
        bindings![MouseButton::Left]
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
        use ::lightyear::prelude::{input::bei::{self, Actions}, Controlled};
        use ::bevy::ecs::{self, system::{self, Query}, query::{self, With}};

        |
            event: ecs::observer::On<ecs::lifecycle::Add, ($owner, Controlled, $(Actions<$context>),*)>,
            $($owners: Query<&Actions<$context>, (With<$owner>, With<Controlled>)>,
                $($actions: Query<(), (With<bei::Action<$action>>, query::Without<bei::Bindings>)>),*
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
    event: On<Add, Player>,
    controlled: Query<&Id, With<Controlled>>,
    mut commands: Commands,
    camera: Query<Entity, With<AimCamera>>,
) {
    commands
        .entity(event.entity)
        .apply_scene(bsn!(
            #Player
            Mesh3d(asset_value(Capsule3d::new(
                PLAYER_RADIUS as f32,
                PLAYER_CAPSULE_LENGTH as f32,
            )))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
            ContextActivity::<Player>::INACTIVE
            Children [
                Mesh3d(asset_value(Cuboid::new(0.1, 0.1, 0.5)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
                Transform::from_xyz(0.0, 0.6, -0.5)
            ]
        ))
        .insert(physics_components());

    if let Ok(id) = controlled.get(event.entity) {
        commands
            .entity(camera.single().expect("multiple aim cameras"))
            .insert((OrbitCamera(*id), SpatialListener::new(PLAYER_RADIUS as f32)));
    }
}

// FIXME: collide with walls
pub fn orbit(
    targets: Query<(&Id, &Position, &Rotation, &Pitch)>,
    cameras: Query<(&mut Transform, &OrbitCamera)>,
) {
    for (mut transform, OrbitCamera(target)) in cameras {
        let target = targets
            .iter()
            .find_map(|(id, pos, rot, pitch)| {
                if **id == **target {
                    Some((pos, rot, pitch))
                } else {
                    None
                }
            })
            .expect("orbit camera target not found");

        *transform = camera_transform(target);
    }
}

pub fn grabber(
    mut button: MessageReader<MouseButtonInput>,
    mut key: MessageReader<KeyboardInput>,
    player: Single<(Entity, &ContextActivity<Player>), (With<Player>, With<Controlled>)>,
    mut options: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    #[allow(clippy::unwrap_used, reason = "there's always only one primary window")]
    let mut options = options.single_mut().unwrap();
    let (entity, looking) = *player;
    let click = button
        .read()
        .any(|b| b.button == MouseButton::Left && b.state == ButtonState::Released);
    let esc = key
        .read()
        .any(|k| k.key_code == KeyCode::Escape && k.state == ButtonState::Pressed);

    if click && esc {
        return;
    }

    if click && !**looking {
        commands.entity(entity).insert(looking.toggled());
        options.visible = false;
        options.grab_mode = CursorGrabMode::Locked;
    } else if esc && **looking {
        commands.entity(entity).insert(looking.toggled());
        options.visible = true;
        options.grab_mode = CursorGrabMode::None;
    }
}
