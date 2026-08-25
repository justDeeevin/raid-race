use crate::component::weapon::PlaceholderGunAssets;
use bevy::{
    asset::AssetServer,
    audio::{AudioPlayer, PlaybackSettings},
    ecs::{
        lifecycle::Add,
        observer::On,
        system::{Commands, Query, Res},
    },
    transform::components::Transform,
};
use raid_race_lib::{event::Attacked, player::weapon::placeholder_gun::PlaceholderGun};
use rand::seq::IndexedRandom;

pub fn placeholder_gun_assets(
    event: On<Add, PlaceholderGun>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    commands.entity(event.entity).insert(PlaceholderGunAssets {
        sounds: vec![assets.load("sound/Geist_wpn_fire_main_01.mp3")],
    });
}

pub fn placeholder_gun_fire(
    event: On<Attacked>,
    mut commands: Commands,
    players: Query<(&Transform, &PlaceholderGunAssets)>,
) {
    if let Ok((transform, assets)) = players.get(**event) {
        commands.spawn((
            *transform,
            #[allow(clippy::unwrap_used, reason = "there's always at least one sound")]
            AudioPlayer(assets.sounds.choose(&mut rand::rng()).unwrap().clone()),
            PlaybackSettings::REMOVE.with_spatial(true),
        ));
    }
}
