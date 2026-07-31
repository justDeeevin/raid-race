use crate::component::{
    OrbitCamera,
    alive::{
        Agility, Cdr, Dps, Health, Mana, Meter,
        player::{Player, PlayerMovable},
    },
    ui::hud::{HealthBar, ManaBar},
};
use avian3d::{
    collision::{
        collider::{Collider, Sensor},
        collision_events::CollisionEventsEnabled,
    },
    dynamics::rigid_body::{LockedAxes, RigidBody},
};
use bevy::{
    asset::Assets,
    camera::Camera3d,
    color::Color,
    ecs::{
        children,
        entity::Entity,
        query::With,
        system::{Commands, Query, ResMut},
    },
    math::{
        Dir3, Vec3,
        primitives::{Capsule3d, Cuboid},
    },
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
    const HEIGHT: f64 = 2.0;
    // -- DON'T CHANGE --
    const CAPSULE_LENGTH: f64 = HEIGHT - (RADIUS * 2.0);
    // -------------------
    const RADIUS: f64 = 0.5;
    const FOOT_HEIGHT: f64 = 0.02;

    let player = commands
        .spawn((
            Mesh3d(meshes.add(Capsule3d::new(RADIUS as f32, CAPSULE_LENGTH as f32))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            RigidBody::Dynamic,
            Collider::capsule(RADIUS, CAPSULE_LENGTH),
            LockedAxes::ROTATION_LOCKED,
            PlayerMovable::default(),
            Health(Meter::new(100)),
            Mana(Meter {
                cap: 40,
                current: 20,
            }),
            Dps(20),
            Agility(10),
            Cdr(0),
            Transform::from_xyz(0.0, CAPSULE_LENGTH as f32 / 2.0, 0.0),
            Player,
            children![
                (
                    Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.5))),
                    MeshMaterial3d(materials.add(Color::WHITE)),
                    Transform::from_xyz(0.0, 0.6, 0.5),
                ),
                (
                    Collider::cylinder(RADIUS, FOOT_HEIGHT),
                    Sensor,
                    CollisionEventsEnabled,
                    Transform::from_xyz(0.0, ((-HEIGHT - FOOT_HEIGHT) / 2.0) as f32, 0.0),
                )
            ],
        ))
        .id();

    const CAMERA_OFFSET: Vec3 = Vec3::new(1.0, 1.0, 0.0);

    commands.spawn((
        Camera3d::default(),
        Transform::default().looking_to(Dir3::Z, Dir3::Y),
        OrbitCamera {
            target: player,
            offset: CAMERA_OFFSET,
        },
    ));
}

pub fn hud(mut commands: Commands, player: Query<Entity, With<Player>>) {
    const N_ROWS: u16 = {
        // -- change this --
        let n = 3;
        // -----------------
        assert!(n % 2 == 1, "N_ROWS must be odd");
        n
    };

    let player = player.single().expect("Could not find just one player");
    commands.spawn((
        // Screen
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Grid,
            grid_template_rows: vec![RepeatedGridTrack::fr(N_ROWS, 1.0)],
            grid_template_columns: vec![RepeatedGridTrack::percent(1, 100.0)],
            ..Default::default()
        },
        children![
            (
                // Crosshair
                Node {
                    grid_row: GridPlacement::start((N_ROWS as i16 / 2) + 1),
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
                // Bottom row
                Node {
                    width: percent(100),
                    display: Display::Grid,
                    grid_row: GridPlacement::start(-2),
                    grid_template_columns: vec![RepeatedGridTrack::fr(3, 1.0)],
                    margin: UiRect::horizontal(vw(3)).with_bottom(vh(5)),
                    ..Default::default()
                },
                children![(
                    // Bars
                    Node {
                        grid_column: GridPlacement::start(1),
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Start,
                        ..Default::default()
                    },
                    children![
                        (
                            // Health
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
                            // Mana
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
