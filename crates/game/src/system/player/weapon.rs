use crate::component::weapon::WeaponAssets;
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
use lightyear::prelude::Controlled;
use raid_race_lib::{
    component::alive::player::weapon::{HeldWeapon, Weapon},
    event::Attacked,
};
use rand::seq::IndexedRandom;

pub fn load_weapon_assets(
    event: On<Add, (Weapon, HeldWeapon)>,
    weapons: Query<&Weapon>,
    held_weapons: Query<&HeldWeapon>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let weapon = match weapons.get(event.entity) {
        Ok(weapon) => weapon,
        Err(_) => match held_weapons.get(event.entity) {
            Ok(HeldWeapon(weapon)) => weapon,
            Err(_) => return,
        },
    };

    match weapon {
        Weapon::PlaceholderGun => {
            commands.entity(event.entity).insert(WeaponAssets {
                sounds: vec![assets.load("sound/Geist_wpn_fire_main_01.mp3")],
            });
        }
    }
}

pub fn fire(
    event: On<Attacked>,
    mut commands: Commands,
    players: Query<(&Transform, &WeaponAssets, Option<&Controlled>)>,
) {
    if let Ok((transform, assets, controlled)) = players.get(**event) {
        let sound_bundle = (
            #[allow(clippy::unwrap_used, reason = "there's always at least one sound")]
            AudioPlayer(assets.sounds.choose(&mut rand::rng()).unwrap().clone()),
            PlaybackSettings::REMOVE.with_spatial(true),
        );
        if controlled.is_some() {
            commands.spawn((*transform, sound_bundle));
        } else {
            let sound = commands.spawn(sound_bundle).id();
            commands.entity(**event).add_child(sound);
        }
    }
}
