use avian3d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::prelude::*;

pub fn shoot(
    space: SpatialQuery,
    source: Entity,
    origin: Vector,
    direction: Dir3,
    distance: Scalar,
) -> Option<Entity> {
    let filter = SpatialQueryFilter::from_excluded_entities([source]);

    space
        .cast_ray(origin, direction, distance, false, &filter)
        .map(|hit| hit.entity)
}
