use std::marker::PhantomData;

use bevy::{
    ecs::{
        entity::Entity,
        event::{EntityEvent, Event},
    },
    prelude::Deref,
};

#[derive(Event)]
pub struct Attacked {
    pub source: Entity,
    // pub target: Entity,
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
