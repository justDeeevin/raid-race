use bevy::{
    ecs::{entity::Entity, resource::Resource},
    prelude::{Deref, DerefMut},
};
use raid_race_lib::message::Input;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Looking(pub bool);

#[derive(Resource, Deref, DerefMut)]
pub struct Me(pub Entity);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct InputState(pub Input);
