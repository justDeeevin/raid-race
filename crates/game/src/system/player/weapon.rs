use crate::component::weapon::WeaponAssets;
use avian3d::physics_transform::Position;
use bevy::prelude::*;
use lightyear::prelude::*;
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

fn my_attack(event: On<Attacked>, attackers: Query<&WeaponAssets>, mut commands: Commands) {
    if let Ok(assets) = attackers.get(**event) {
        let sound = commands
            .spawn((assets.sound_bundle(), Transform::default()))
            .id();

        commands.entity(**event).add_child(sound);
    }
}

fn not_my_attack(
    event: On<RemoteEvent<Attacked>>,
    mut commands: Commands,
    attackers: Query<(&WeaponAssets, &Position)>,
) {
    if let Ok((assets, position)) = attackers.get(*event.trigger) {
        commands.spawn((
            assets.sound_bundle(),
            Transform::from_translation(position.as_vec3()),
        ));
    }
}

pub fn plugin(app: &mut App) {
    app.add_observer(not_my_attack)
        .add_observer(my_attack)
        .add_observer(load_weapon_assets);
}
