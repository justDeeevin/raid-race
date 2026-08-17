use crate::system::{
    client::{self, ConnectCommand, DisconnectCommand, TokenTask},
    player::{self, WhoAmI, add_bindings_on_owner_spawn},
    ui::hud,
};
use bevy::{
    app::{App, Update},
    ecs::{
        query::With,
        schedule::{
            IntoScheduleConfigs, SystemCondition,
            common_conditions::{on_message, resource_exists},
        },
        world::World,
    },
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput},
};
use bevy_console::AddConsoleCommand;
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin,
    bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext},
    bevy_inspector,
    egui::{ScrollArea, Window},
};
use lightyear::prelude::client::ClientPlugins;
use raid_race_lib::{
    TICK_PERIOD,
    component::alive::player::Player,
    input::{Ability, Jump, Look, Walk},
};

pub fn hud(app: &mut App) {
    app.add_systems(
        Update,
        (
            hud::health_bar,
            hud::mana_bar,
            hud::ability_cooldown::<1>,
            hud::ability_cooldown::<2>,
            hud::ability_cooldown::<3>,
            hud::ability_cooldown::<4>,
            hud::ability_cooldown::<5>,
        ),
    )
    .add_observer(hud::spawn)
    .add_observer(hud::despawn)
    .add_observer(hud::add_poison)
    .add_observer(hud::remove_poison);
}

pub fn client(app: &mut App) {
    app.add_plugins((
        ClientPlugins {
            tick_duration: TICK_PERIOD,
        },
        raid_race_lib::plugin,
    ))
    .add_systems(
        Update,
        client::wait_for_token.run_if(resource_exists::<TokenTask>),
    )
    .add_console_command::<ConnectCommand, _>(client::connect_command)
    .add_console_command::<DisconnectCommand, _>(client::disconnect_command);
}

pub fn player(app: &mut App) {
    app.add_systems(
        Update,
        (
            player::orbit,
            player::grabber
                .run_if(on_message::<MouseButtonInput>.or_eager(on_message::<KeyboardInput>)),
        ),
    )
    .add_observer(player::spawn)
    .add_observer(player::add_bindings_on_action_spawn::<Walk, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Look, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Jump, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Ability<1>, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Ability<2>, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Ability<3>, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Ability<4>, Player, Player>)
    .add_observer(player::add_bindings_on_action_spawn::<Ability<5>, Player, Player>)
    .add_observer(add_bindings_on_owner_spawn!(Player {
        players: Player[
            walks: Walk,
            looks: Look,
            jumps: Jump,
            ones: Ability<1>,
            twos: Ability<2>,
            threes: Ability<3>,
            fours: Ability<4>,
            fives: Ability<5>
        ],
    }))
    .add_console_command::<WhoAmI, _>(player::whoami);
}

#[allow(unused)]
pub fn inspector(app: &mut App) {
    fn ui(world: &mut World) {
        let Ok(mut ctx) = world
            .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
            .single(world)
            .cloned()
        else {
            return;
        };

        Window::new("World Inspector")
            .default_size((400.0, 300.0))
            .show(ctx.get_mut(), |ui| {
                ScrollArea::both().show(ui, |ui| {
                    bevy_inspector::ui_for_world(world, ui);
                    ui.allocate_space(ui.available_size());
                })
            });
    }

    app.add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(EguiPrimaryContextPass, ui);
}
