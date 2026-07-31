use bevy::{
    ecs::{entity::Entity, resource::Resource},
    prelude::{Deref, DerefMut},
};
use raid_race_lib::message::{Buttons, Input};

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Looking(pub bool);

#[derive(Resource, Deref, DerefMut)]
pub struct Me(pub Entity);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Inputs(pub Vec<Input>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct InputState(pub Buttons);
