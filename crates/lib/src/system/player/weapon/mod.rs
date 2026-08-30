pub mod placeholder_gun;

use crate::{
    component::alive::player::{
        Pitch,
        weapon::{HeldWeapon, Weapon},
    },
    event::{Attacked, Hit},
    system::player::camera_transform,
};
use avian3d::{math::Scalar, prelude::*};
use bevy::prelude::*;

pub fn attack(
    event: On<Attacked>,
    space: SpatialQuery,
    players: Query<(&HeldWeapon, &Position, &Rotation, &Pitch)>,
    mut commands: Commands,
) {
    const MAX_DISTANCE: Scalar = 200.0;

    let Ok((HeldWeapon(weapon), position, rotation, pitch)) = players.get(**event) else {
        return;
    };
    let camera = camera_transform((position, rotation, pitch));
    let filter = SpatialQueryFilter::from_excluded_entities([**event]);

    if let Some(camera_hit) = space.cast_ray(
        camera.translation.as_dvec3(),
        camera.forward(),
        MAX_DISTANCE,
        false,
        &filter,
    ) && let Ok(dir) = Dir3::new(
        (camera.translation + (camera.forward() * camera_hit.distance as f32)) - position.as_vec3(),
    ) && let Some(target) = match weapon {
        Weapon::PlaceholderGun => {
            placeholder_gun::shoot(space, **event, **position, dir, camera_hit.distance)
        }
    } {
        commands.trigger(Hit {
            source: **event,
            target,
        })
    }
}
