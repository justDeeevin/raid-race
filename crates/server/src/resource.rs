use bevy::{
    ecs::{entity::Entity, resource::Resource},
    prelude::{Deref, DerefMut},
};
use naia_bevy_server::{RoomKey, UserKey};
use raid_race_lib::message::Buttons;
use std::collections::HashMap;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Room(pub Option<RoomKey>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct UserEntities(HashMap<UserKey, Entity>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct Inputs(pub HashMap<Entity, Buttons>);
