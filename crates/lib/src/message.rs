use bevy::ecs::entity::Entity;
use naia_bevy_shared::{Message, Serde};

pub use naia_bevy_shared::Message as Trait;

#[derive(Message, Default)]
pub struct Input {
    pub buttons: Buttons,
}

#[derive(Serde, Clone, Copy, PartialEq, Eq, Default)]
pub struct Buttons(u8);

bitflags::bitflags! {
    impl Buttons: u8 {
        const FORWARD = 1;
        const RIGHT = 1 << 1;
        const BACKWARD = 1 << 2;
        const LEFT = 1 << 3;
        const JUMP = 1 << 4;
    }
}

#[derive(Message)]
pub struct Auth;

#[derive(Message)]
pub struct You(pub EntityBits);

#[derive(Serde, Clone, Copy, PartialEq, Eq)]
pub struct EntityBits(u64);

impl From<Entity> for EntityBits {
    fn from(entity: Entity) -> Self {
        Self(entity.to_bits())
    }
}

impl From<EntityBits> for Entity {
    fn from(entity: EntityBits) -> Self {
        Self::from_bits(entity.0)
    }
}
