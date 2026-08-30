pub mod weapon;

use crate::component::{AimCamera, OrbitCamera};
use avian3d::physics_transform::{Position, Rotation};
use bevy::{
    audio::Volume,
    input::{ButtonState, keyboard::KeyboardInput, mouse::MouseButtonInput},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use bevy_console::{
    AddConsoleCommand, ConsoleCommand,
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
    system::player::{PLAYER_CAPSULE_LENGTH, PLAYER_RADIUS, camera_transform, physics_components},
};

#[derive(Parser, ConsoleCommand, Deref)]
#[command(name = "volume")]
pub struct VolumeCommand {
    #[arg()]
    /// The linear volume—1.0=100%
    pub value: f32,
}

fn volume(mut command: ConsoleCommand<VolumeCommand>, mut volume: ResMut<GlobalVolume>) {
    if let Some(Ok(value)) = command.take() {
        volume.volume = Volume::Linear(*value);
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "whoami")]
/// Print the entity ID of the currently controlled player
pub struct WhoAmI;

fn whoami(
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
trait Binds: InputAction {
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
fn add_bindings_on_action_spawn<A: Binds, Context: Component, Owner: Component>(
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

fn spawn(
    event: On<Add, Player>,
    controlled: Query<(), With<Controlled>>,
    mut commands: Commands,
    camera: Query<Entity, With<AimCamera>>,
) {
    const NOSE_LENGTH: f32 = PLAYER_RADIUS as f32 * 1.5;

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
                Mesh3d(asset_value(Cuboid::new(0.1, 0.1, NOSE_LENGTH)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::WHITE))
                Transform::from_xyz(0.0, PLAYER_CAPSULE_LENGTH as f32 / 2.0, -NOSE_LENGTH)
            ]
        ))
        .insert(physics_components());

    if controlled.get(event.entity).is_ok() {
        // TODO: this should just be a resource
        commands
            .entity(camera.single().expect("multiple aim cameras"))
            .insert((
                OrbitCamera(event.entity),
                SpatialListener::new(-PLAYER_RADIUS as f32),
            ));
    }
}

// FIXME: collide with walls
fn orbit(
    targets: Query<(&Position, &Rotation, &Pitch)>,
    cameras: Query<(&mut Transform, &OrbitCamera)>,
) {
    for (mut transform, OrbitCamera(target)) in cameras {
        if let Ok(target) = targets.get(*target) {
            *transform = camera_transform(target);
        }
    }
}

fn grabber(
    mut button: MessageReader<MouseButtonInput>,
    mut key: MessageReader<KeyboardInput>,
    player: Single<(Entity, &ContextActivity<Player>), (With<Player>, With<Controlled>)>,
    mut options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    let keys = key.read().collect::<Vec<_>>();
    let (entity, looking) = *player;
    let enter = button
        .read()
        .any(|b| b.button == MouseButton::Left && b.state == ButtonState::Released)
        || keys
            .iter()
            .any(|k| k.key_code == KeyCode::Tab && k.state == ButtonState::Released);
    let exit = keys.iter().any(|k| {
        (k.key_code == KeyCode::Escape && k.state == ButtonState::Pressed)
            || (k.key_code == KeyCode::Tab && k.state == ButtonState::Pressed)
    });

    if enter && exit {
        return;
    }

    if enter && !**looking {
        commands.entity(entity).insert(looking.toggled());
        options.visible = false;
        options.grab_mode = CursorGrabMode::Locked;
    } else if exit && **looking {
        commands.entity(entity).insert(looking.toggled());
        options.visible = true;
        options.grab_mode = CursorGrabMode::None;
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            orbit,
            grabber.run_if(on_message::<MouseButtonInput>.or_eager(on_message::<KeyboardInput>)),
        ),
    )
    .add_observer(spawn)
    .add_observer(add_bindings_on_action_spawn::<Walk, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Look, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Jump, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Ability<1>, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Ability<2>, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Ability<3>, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Ability<4>, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Ability<5>, Player, Player>)
    .add_observer(add_bindings_on_action_spawn::<Attack, Player, Player>)
    .add_observer(add_bindings_on_owner_spawn!(Player {
        players: Player[
            walks: Walk,
            looks: Look,
            jumps: Jump,
            ones: Ability<1>,
            twos: Ability<2>,
            threes: Ability<3>,
            fours: Ability<4>,
            fives: Ability<5>,
            attacks: Attack,
        ],
    }))
    .add_console_command::<WhoAmI, _>(whoami)
    .add_console_command::<VolumeCommand, _>(volume)
    .add_plugins(weapon::plugin);
}
