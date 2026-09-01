use crate::component::ui::hud::{Ability, HealthBar, ManaBar};
use avian3d::parry::either::Either;
use bevy::{
    app::{App, Update},
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
        template::FromTemplate,
        world::Ref,
    },
    picking::events::{Enter, Leave, Pointer},
    prelude::Deref,
    scene::{CommandsSceneExt, Scene, bsn},
    text::{FontSize, Justify, TextColor, TextFont, TextLayout},
    ui::{
        AlignItems, BackgroundColor, BorderColor, Display, FlexDirection, GridPlacement,
        JustifyContent, Node, RepeatedGridTrack, UiRect, percent, px, vh, vw, widget::Text,
    },
};
use lightyear::{
    connection::client::Disconnected,
    prelude::{Controlled, input::bei::InputAction},
};
use raid_race_lib::{
    component::alive::{
        Health, Mana,
        player::{
            Player,
            character::{Character, Cooldowns},
        },
        status::Poison,
    },
    input,
};

fn health_bar(
    bar: Query<(&mut Node, &HealthBar, &mut Text)>,
    health: Query<&Health, Changed<Health>>,
) {
    for (mut node, HealthBar(entity), mut text) in bar {
        if let Ok(health) = health.get(*entity) {
            node.width = percent(100.0 * health.current as f32 / health.cap as f32);
            **text = format!("{}/{}", health.current, health.cap);
        }
    }
}

fn add_poison(event: On<Add, Poison>, mut color: Query<(&HealthBar, &mut BackgroundColor)>) {
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

fn remove_poison(event: On<Remove, Poison>, mut color: Query<(&HealthBar, &mut BackgroundColor)>) {
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

fn mana_bar(bar: Query<(&mut Node, &ManaBar, &mut Text)>, mana: Query<Ref<Mana>>) {
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

fn ability_cooldown<const N: usize>(
    hud: Query<(&mut Text, &Ability<N>)>,
    cd: Query<(&Cooldowns, &Character), (With<Controlled>, Changed<Cooldowns>)>,
) where
    input::Ability<N>: InputAction,
{
    for (mut text, Ability(target)) in hud {
        if let Ok((cooldowns, character)) = cd.get(*target) {
            match &cooldowns[N - 1] {
                Either::Left(cd) => {
                    if cd.is_finished() {
                        // FIXME: shrink text size to fit
                        **text = character.data.ability(N).to_string();
                    } else {
                        **text = cd.remaining_secs().ceil().to_string();
                    }
                }
                Either::Right(ready) => {
                    if *ready {
                        **text = character.data.ability(N).to_string();
                    } else {
                        **text = "Not ready".to_string();
                    }
                }
            }
        }
    }
}

fn ability_description_popup(
    event: On<Pointer<Enter>>,
    boxes: Query<&Children>,
    abilities: Query<&Description>,
    mut descriptions: Query<&mut Node>,
) {
    if let Ok(children) = boxes.get(event.entity)
        && let Some(child) = children.first()
        && let Ok(Description(entity)) = abilities.get(*child)
        && let Ok(mut node) = descriptions.get_mut(*entity)
    {
        node.display = Display::Flex;
    }
}

fn ability_description_popup_exit(
    event: On<Pointer<Leave>>,
    boxes: Query<&Children>,
    abilities: Query<&Description>,
    mut descriptions: Query<&mut Node>,
) {
    if let Ok(children) = boxes.get(event.entity)
        && let Some(child) = children.first()
        && let Ok(Description(entity)) = abilities.get(*child)
        && let Ok(mut node) = descriptions.get_mut(*entity)
    {
        node.display = Display::None;
    }
}

fn ability_description<const N: usize>(
    abilities: Query<(&Ability<N>, &Description)>,
    mut descriptions: Query<&mut Text>,
    characters: Query<&Character, Changed<Character>>,
) where
    input::Ability<N>: InputAction,
{
    for (Ability(target), Description(entity)) in abilities {
        if let Ok(character) = characters.get(*target)
            && let Ok(mut text) = descriptions.get_mut(*entity)
        {
            **text = character.data.ability(N).description();
        }
    }
}

#[derive(Component, Default, Clone, Copy)]
struct HudRoot;

fn spawn(
    event: On<Add, (Player, Controlled)>,
    me: Query<(), (With<Health>, With<Mana>, With<Player>, With<Controlled>)>,
    hud: Query<Entity, With<HudRoot>>,
    mut commands: Commands,
) {
    if hud.is_empty() && me.get(event.entity).is_ok() {
        commands.spawn_scene(scene(event.entity));
    }
}

fn despawn(_: On<Add, Disconnected>, hud: Single<Entity, With<HudRoot>>, mut commands: Commands) {
    commands.entity(*hud).despawn();
}

fn scene(target: Entity) -> impl Scene {
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
                    (
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
                                        Text
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
                                        Text
                                        TextFont {
                                            font_size: FontSize::Px(15.0),
                                        }
                                    )
                                ]
                            )
                        ]
                    ),
                    (
                        #Abilities
                        Node {
                            grid_column: GridPlacement::start(2),
                            justify_content: JustifyContent::Center,
                            column_gap: px(10),
                        }
                        Children [
                            ability::<1>(target),
                            ability::<2>(target),
                            ability::<3>(target),
                            ability::<4>(target),
                            ability::<5>(target),
                        ]
                    )
                ]
            )
        ]
    }
}

#[derive(Component, Deref, FromTemplate)]
#[relationship_target(relationship = DescriptionOf)]
struct Description(Entity);

#[derive(Component, Deref, FromTemplate)]
#[relationship(relationship_target = Description)]
struct DescriptionOf(Entity);

fn ability<const N: usize>(target: Entity) -> impl Scene
where
    input::Ability<N>: InputAction,
{
    let font_size = px(15);

    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
        }
        Children [
            (
                #Description
                Node {
                    padding: UiRect::all(font_size * 0.75),
                    display: Display::None
                }
                BackgroundColor(Color::BLACK)
                Text
                TextLayout::justify(Justify::Center)
                TextFont {
                    font_size,
                }
                TextColor(Color::WHITE)
                DescriptionOf(#Ability)
            ),
            (
                Node {
                    height: px(100),
                    width: px(100),
                    border: UiRect::all(px(4)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                }
                BackgroundColor(Color::WHITE)
                BorderColor::all(Color::BLACK)
                Description(#Description)
                Children [
                    #Ability
                    Ability::<N>(target)
                    Text
                    TextLayout::justify(Justify::Center)
                    TextColor(Color::BLACK)
                ]
            ),
        ]
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            health_bar,
            mana_bar,
            ability_cooldown::<1>,
            ability_cooldown::<2>,
            ability_cooldown::<3>,
            ability_cooldown::<4>,
            ability_cooldown::<5>,
            ability_description::<1>,
            ability_description::<2>,
            ability_description::<3>,
            ability_description::<4>,
            ability_description::<5>,
        ),
    )
    .add_observer(spawn)
    .add_observer(despawn)
    .add_observer(add_poison)
    .add_observer(remove_poison)
    .add_observer(ability_description_popup)
    .add_observer(ability_description_popup_exit);
}
