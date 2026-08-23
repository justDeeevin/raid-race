use bevy::{
    ecs::{
        entity::Entity,
        event::{EntityEvent, Event},
    },
    prelude::Deref,
};
use std::marker::PhantomData;

#[derive(EntityEvent, Deref)]
pub struct Attacked(pub Entity);

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
