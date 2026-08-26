use crate::component::weapon::WeaponAssets;
use bevy::prelude::*;
use lightyear::prelude::MessageReceiver;
use raid_race_lib::{
    component::alive::player::weapon::{HeldWeapon, Weapon},
    event::Attacked,
};

fn load_weapon_assets(
    event: On<Add, (Weapon, HeldWeapon)>,
    weapons: Query<&Weapon>,
    held_weapons: Query<&HeldWeapon>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let Ok(weapon) = weapons
        .get(event.entity)
        .or_else(|_| held_weapons.get(event.entity).map(|w| &w.0))
    else {
        return;
    };

    match weapon {
        Weapon::PlaceholderGun => {
            commands.entity(event.entity).insert(WeaponAssets {
                sounds: vec![assets.load("sound/Geist_wpn_fire_main_01.mp3")],
            });
        }
    }
}

fn my_attack(event: On<Attacked>, mut commands: Commands, attackers: Query<&WeaponAssets>) {
    if let Ok(assets) = attackers.get(**event) {
        let sound = commands
            .spawn((assets.sound_bundle(), Transform::default()))
            .id();
        commands.entity(**event).add_child(sound);
    }
}

fn not_my_attack(
    mut messages: Single<&mut MessageReceiver<Attacked>>,
    mut commands: Commands,
    attackers: Query<(&Transform, &WeaponAssets)>,
) {
    for event in messages.receive() {
        if let Ok((transform, assets)) = attackers.get(*event) {
            commands.spawn((assets.sound_bundle(), *transform));
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, not_my_attack)
        .add_observer(my_attack)
        .add_observer(load_weapon_assets);
}
