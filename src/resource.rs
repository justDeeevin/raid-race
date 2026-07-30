use bevy::{
    ecs::resource::Resource,
    prelude::{Deref, DerefMut},
};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Looking(pub bool);
