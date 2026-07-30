use crate::component::{
    alive::{
        Agility, Cdr, Dps, Health, Mana, Meter,
        player::{Player, PlayerMovable},
    },
    ui::{HealthBar, ManaBar},
};
use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LockedAxes, RigidBody},
};
use bevy::{
    asset::Assets,
    color::Color,
    ecs::{
        children,
        entity::Entity,
        query::With,
        system::{Commands, Query, ResMut},
    },
    math::primitives::Cuboid,
    mesh::{Mesh, Mesh3d},
    pbr::{MeshMaterial3d, StandardMaterial},
    text::{FontSize, TextFont},
    transform::components::Transform,
    ui::{
        AlignItems, BackgroundColor, BorderColor, Display, FlexDirection, GridPlacement,
        JustifyContent, Node, RepeatedGridTrack, UiRect, percent, px, vh, vw, widget::Text,
    },
};

pub fn player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        RigidBody::Dynamic,
        Collider::cuboid(1.0, 1.0, 1.0),
        LockedAxes::ROTATION_LOCKED,
        PlayerMovable {
            airborne: true,
            ..Default::default()
        },
        Health(Meter::new(100)),
        Mana(Meter {
            cap: 40,
            current: 20,
        }),
        Dps(20),
        Agility(10),
        Cdr(0),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Player,
    ));
}

pub fn hud(mut commands: Commands, player: Query<Entity, With<Player>>) {
    let player = player.single().expect("Could not find just one player");
    commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::End,
            ..Default::default()
        },
        children![(
            Node {
                width: percent(100),
                display: Display::Grid,
                grid_template_columns: vec![RepeatedGridTrack::fr(3, 1.0)],
                margin: UiRect::horizontal(vw(3)).with_bottom(vh(5)),
                ..Default::default()
            },
            children![(
                Node {
                    grid_column: GridPlacement::start(1),
                    flex_direction: FlexDirection::ColumnReverse,
                    justify_content: JustifyContent::Start,
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
                            BackgroundColor(Color::srgb_u8(180, 0, 0)),
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
        ),],
    ));
}
