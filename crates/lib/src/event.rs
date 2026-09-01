use bevy::{ecs::entity::MapEntities, prelude::*};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(EntityEvent, Deref, Serialize, Deserialize, Clone)]
pub struct Attacked(pub Entity);

impl MapEntities for Attacked {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.0.map_entities(entity_mapper);
    }
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Slotted {
    pub entity: Entity,
    pub index: usize,
}

impl MapEntities for Slotted {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        self.entity.map_entities(entity_mapper);
    }
}

#[derive(Resource, Event, Serialize, Deserialize, Deref, DerefMut, Clone, Copy, Default)]
pub struct NoCD(pub bool);
