use avian3d::{
    math::Vector,
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::{ecs::entity::Entity, math::Dir3};

pub fn shoot(
    space: SpatialQuery,
    source: Entity,
    origin: Vector,
    direction: Dir3,
    distance: f64,
) -> Option<Entity> {
    let filter = SpatialQueryFilter::from_excluded_entities([source]);

    space
        .cast_ray(origin, direction, distance, false, &filter)
        .map(|hit| hit.entity)
}
