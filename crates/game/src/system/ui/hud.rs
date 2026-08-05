use crate::component::ui::hud::{HealthBar, ManaBar};
use bevy::{
    color::Color,
    ecs::{
        change_detection::DetectChanges,
        children,
        component::Component,
        entity::Entity,
        observer::On,
        system::{Commands, Query},
        world::Ref,
    },
    text::{FontSize, TextFont},
    ui::{
        AlignItems, BackgroundColor, BorderColor, Display, FlexDirection, GridPlacement,
        JustifyContent, Node, RepeatedGridTrack, UiRect, percent, px, vh, vw, widget::Text,
    },
};
use raid_race_lib::component::alive::{
    Health, Mana,
    status::{Poison, PoisonRemoved},
};

pub fn health_bar(
    bar: Query<(&mut Node, &HealthBar, &mut Text, &mut BackgroundColor)>,
    health: Query<(Ref<Health>, Option<Ref<Poison>>)>,
) {
    for (mut node, HealthBar(entity), mut text, mut color) in bar {
        let (health, poison) = health.get(*entity).expect("Health bar target not found");
        if health.is_changed() {
            node.width = percent(100.0 * health.current as f32 / health.cap as f32);
            **text = format!("{}/{}", health.current, health.cap);
        }
        if let Some(poison) = poison
            && poison.is_changed()
        {
            **color = HealthBar::PURPLE;
        }
    }
}

pub fn remove_poison(
    event: On<PoisonRemoved>,
    mut color: Query<(&HealthBar, &mut BackgroundColor)>,
) {
    let Some(mut color) = color.iter_mut().find_map(|(entity, color)| {
        if event.from == entity.0 {
            Some(color)
        } else {
            None
        }
    }) else {
        return;
    };

    **color = HealthBar::RED;
}

pub fn mana_bar(bar: Query<(&mut Node, &ManaBar, &mut Text)>, mana: Query<Ref<Mana>>) {
    for (mut node, ManaBar(entity), mut text) in bar {
        let mana = mana.get(*entity).expect("Mana bar target not found");
        if mana.is_changed() {
            node.width = percent(100.0 * mana.current as f32 / mana.cap as f32);
            text.0 = format!("{}/{}", mana.current, mana.cap);
        }
    }
}

#[derive(Component)]
pub struct HudRoot;

pub fn spawn(mut commands: Commands, player: Entity) {
    commands.spawn((
        HudRoot,
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Grid,
            grid_template_rows: vec![RepeatedGridTrack::fr(3, 1.0)],
            grid_template_columns: vec![RepeatedGridTrack::percent(1, 100.0)],
            ..Default::default()
        },
        children![
            (
                Node {
                    grid_row: GridPlacement::start(2),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                children![(
                    Node {
                        height: px(10),
                        width: px(10),
                        ..Default::default()
                    },
                    BackgroundColor(Color::BLACK)
                )]
            ),
            (
                Node {
                    width: percent(100),
                    display: Display::Grid,
                    grid_row: GridPlacement::start(3),
                    grid_template_columns: vec![RepeatedGridTrack::fr(3, 1.0)],
                    margin: UiRect::horizontal(vw(3)).with_bottom(vh(5)),
                    ..Default::default()
                },
                children![(
                    Node {
                        grid_column: GridPlacement::start(1),
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Start,
                        ..Default::default()
                    },
                    children![
                        (
                            Node {
                                height: vh(4),
                                width: percent(100),
                                border: UiRect::all(px(4)),
                                ..Default::default()
                            },
                            BorderColor::all(Color::BLACK),
                            children![(
                                Node {
                                    height: percent(100),
                                    width: percent(100),
                                    ..Default::default()
                                },
                                BackgroundColor(HealthBar::RED),
                                HealthBar(player),
                                Text::default(),
                            ),]
                        ),
                        (
                            Node {
                                height: vh(2),
                                width: percent(80),
                                border: UiRect::all(px(3)),
                                ..Default::default()
                            },
                            BorderColor::all(Color::BLACK),
                            children![(
                                Node {
                                    height: percent(100),
                                    width: percent(100),
                                    ..Default::default()
                                },
                                BackgroundColor(Color::srgb_u8(0, 0, 255)),
                                ManaBar(player),
                                Text::default(),
                                TextFont {
                                    font_size: FontSize::Px(15.0),
                                    ..Default::default()
                                }
                            )]
                        )
                    ]
                )]
            ),
        ],
    ));
}
