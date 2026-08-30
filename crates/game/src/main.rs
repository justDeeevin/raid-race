mod component;
mod system;

use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Startup},
    camera::Camera3d,
    ecs::{query::With, system::Commands, world::World},
    log::LogPlugin,
    math::{Dir3, Vec3},
    transform::components::Transform,
};
use bevy_console::ConsolePlugin;
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin,
    bevy_egui::{EguiContext, EguiPrimaryContextPass, PrimaryEguiContext},
    bevy_inspector,
    egui::{ScrollArea, Window},
};
use component::AimCamera;
use system::*;

fn main() {
    App::default()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                custom_layer: bevy_console::make_layer,
                ..Default::default()
            }),
            ConsolePlugin,
            client::plugin,
            player::plugin,
            ui::hud::plugin,
            inspector,
            // avian3d::debug_render::PhysicsDebugPlugin,
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 100.0, 100.0).looking_at(Vec3::ZERO, Dir3::Y),
                AimCamera,
            ));
        })
        .run();
}

#[allow(unused)]
fn inspector(app: &mut App) {
    fn ui(world: &mut World) {
        if let Ok(mut ctx) = world
            .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
            .single(world)
            .cloned()
        {
            Window::new("World Inspector")
                .default_size((400.0, 300.0))
                .show(ctx.get_mut(), |ui| {
                    ScrollArea::both().show(ui, |ui| {
                        bevy_inspector::ui_for_world(world, ui);
                        ui.allocate_space(ui.available_size());
                    })
                });
        }
    }

    app.add_plugins(DefaultInspectorConfigPlugin)
        .add_systems(EguiPrimaryContextPass, ui);
}
