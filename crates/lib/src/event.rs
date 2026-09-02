use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(EntityEvent, MapEntities, Deref, Serialize, Deserialize, Clone)]
pub struct Attacked(#[entities] pub Entity);

#[derive(Event)]
pub struct Hit {
    pub source: Entity,
    pub target: Entity,
}

#[derive(EntityEvent, Deref)]
pub struct Cast<T> {
    #[deref]
    pub entity: Entity,
    _marker: PhantomData<T>,
}

impl<T> Cast<T> {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

#[derive(MapEntities, Serialize, Deserialize, Clone)]
pub struct Slotted {
    #[entities]
    pub entity: Entity,
    pub index: usize,
}

#[derive(Resource, Serialize, Deserialize, Deref, DerefMut, Default)]
pub struct NoCD(pub bool);
