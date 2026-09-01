use avian3d::{math::RVector, prelude::*};
use bevy::prelude::*;

pub fn shoot(
    space: SpatialQuery,
    source: Entity,
    origin: RVector,
    direction: Dir3,
    distance: f32,
) -> Option<Entity> {
    let filter = SpatialQueryFilter::from_excluded_entities([source]);

    space
        .cast_ray(origin, direction, distance, false, &filter)
        .map(|hit| hit.entity)
}
