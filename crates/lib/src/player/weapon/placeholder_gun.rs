use avian3d::{
    physics_transform::{Position, Rotation},
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::{
    ecs::{
        component::Component,
        observer::On,
        query::With,
        system::{Commands, Query},
    },
    math::Dir3,
};
use serde::{Deserialize, Serialize};

use crate::{
    component::alive::player::Pitch,
    event::{Attacked, Hit},
    player::camera_transform,
};

#[derive(Component, Serialize, Deserialize)]
pub struct PlaceholderGun;

pub fn shoot(
    event: On<Attacked>,
    space: SpatialQuery,
    players: Query<(&Position, &Rotation, &Pitch), With<PlaceholderGun>>,
    mut commands: Commands,
) {
    const MAX_DISTANCE: f64 = 200.0;

    let Ok(target) = players.get(**event) else {
        return;
    };
    let camera = camera_transform(target);
    let filter = SpatialQueryFilter::from_excluded_entities([**event]);

    if let Some(camera_hit) = space.cast_ray(
        camera.translation.as_dvec3(),
        camera.forward(),
        MAX_DISTANCE,
        false,
        &filter,
    ) && let Ok(dir) = Dir3::new(
        (camera.translation + (camera.forward() * camera_hit.distance as f32)) - target.0.as_vec3(),
    ) && let Some(hit) = space.cast_ray(**target.0, dir, MAX_DISTANCE, false, &filter)
        && hit.entity == camera_hit.entity
    {
        tracing::info!("hit");
        commands.trigger(Hit {
            source: **event,
            target: hit.entity,
        });
    }
}
