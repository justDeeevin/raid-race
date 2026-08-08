use crate::component::ui::hud::{HealthBar, ManaBar};
use bevy::{
    color::Color,
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        hierarchy::Children,
        lifecycle::{Add, Remove},
        observer::On,
        query::{Changed, With},
        system::{Commands, Query, Single},
        world::Ref,
    },
    scene::{CommandsSceneExt, Scene, bsn},
    text::{FontSize, TextFont},
    ui::{
        AlignItems, BackgroundColor, BorderColor, Display, FlexDirection, GridPlacement,
        JustifyContent, Node, RepeatedGridTrack, UiRect, percent, px, vh, vw, widget::Text,
    },
};
use lightyear::{connection::client::Disconnected, prelude::Controlled};
use raid_race_lib::component::alive::{Health, Mana, player::Player, status::Poison};

pub fn health_bar(
    bar: Query<(&mut Node, &HealthBar, &mut Text)>,
    health: Query<&Health, Changed<Health>>,
) {
    for (mut node, HealthBar(entity), mut text) in bar {
        if let Ok(health) = health.get(*entity) {
            tracing::info!("redrawing health bar");
            node.width = percent(100.0 * health.current as f32 / health.cap as f32);
            **text = format!("{}/{}", health.current, health.cap);
        }
    }
}

pub fn add_poison(event: On<Add, Poison>, mut color: Query<(&HealthBar, &mut BackgroundColor)>) {
    if let Some(mut color) = color.iter_mut().find_map(|(entity, color)| {
        if event.entity == **entity {
            Some(color)
        } else {
            None
        }
    }) {
        **color = HealthBar::PURPLE;
    }
}

pub fn remove_poison(
    event: On<Remove, Poison>,
    mut color: Query<(&HealthBar, &mut BackgroundColor)>,
) {
    if let Some(mut color) = color.iter_mut().find_map(|(entity, color)| {
        if event.entity == **entity {
            Some(color)
        } else {
            None
        }
    }) {
        **color = HealthBar::RED;
    }
}

pub fn mana_bar(bar: Query<(&mut Node, &ManaBar, &mut Text)>, mana: Query<Ref<Mana>>) {
    for (mut node, ManaBar(entity), mut text) in bar {
        let Ok(mana) = mana.get(*entity) else {
            continue;
        };
        if mana.is_changed() {
            node.width = percent(100.0 * mana.current as f32 / mana.cap as f32);
            text.0 = format!("{}/{}", mana.current, mana.cap);
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct HudRoot;

pub fn spawn(
    event: On<Add, (Player, Controlled)>,
    me: Query<(), (With<Health>, With<Mana>, With<Player>, With<Controlled>)>,
    hud: Query<Entity, With<HudRoot>>,
    mut commands: Commands,
) {
    if hud.is_empty() && me.get(event.entity).is_ok() {
        commands.spawn_scene(scene(event.entity));
    }
}

pub fn despawn(
    _: On<Add, Disconnected>,
    hud: Single<Entity, With<HudRoot>>,
    mut commands: Commands,
) {
    commands.entity(*hud).despawn();
}

pub fn scene(target: Entity) -> impl Scene {
    bsn! {
        #Hud
        HudRoot
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Grid,
            grid_template_rows: vec![RepeatedGridTrack::fr(3, 1.0)],
            grid_template_columns: vec![RepeatedGridTrack::percent(1, 100.0)],
        }
        Children [
            (
                #Crosshair
                Node {
                    grid_row: GridPlacement::start(2),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                Children [
                    Node {
                        height: px(10),
                        width: px(10),
                    }
                    BackgroundColor(Color::BLACK)
                ]
            ),
            (
                #HudBottom
                Node {
                    width: percent(100),
                    display: Display::Grid,
                    grid_row: GridPlacement::start(3),
                    grid_template_columns: vec![RepeatedGridTrack::fr(3, 1.0)],
                    margin: UiRect {
                        left: vw(3),
                        right: vw(3),
                        bottom: vh(5),
                    },
                }
                Children [
                    #Bars
                    Node {
                        grid_column: GridPlacement::start(1),
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Start,
                    }
                    Children [
                        (
                            #HealthBar
                            Node {
                                height: vh(4),
                                width: percent(100),
                                border: UiRect::all(px(4)),
                            }
                            BorderColor::all(Color::BLACK)
                            Children [
                                (
                                    Node {
                                        height: percent(100),
                                        width: percent(100),
                                    }
                                    BackgroundColor(HealthBar::RED)
                                    HealthBar(target)
                                    Text::default()
                                )
                            ]
                        ),
                        (
                            #ManaBar
                            Node {
                                height: vh(2),
                                width: percent(80),
                                border: UiRect::all(px(3)),
                            }
                            BorderColor::all(Color::BLACK)
                            Children [
                                (
                                    Node {
                                        height: percent(100),
                                        width: percent(100),
                                    }
                                    BackgroundColor(Color::srgb_u8(0, 0, 255))
                                    ManaBar(target)
                                    Text::default()
                                    TextFont {
                                        font_size: FontSize::Px(15.0),
                                    }
                                )
                            ]
                        )
                    ]
                ]
            )
        ]
    }
}
