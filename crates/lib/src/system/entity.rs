#[cfg(feature = "server")]
use crate::component::alive::{Agility, Cdr, Defense, Dps, Health, Luck, Mana};
use avian3d::{
    collision::collider::Collider,
    dynamics::rigid_body::{LockedAxes, RigidBody},
};
use bevy::{ecs::bundle::Bundle, transform::components::Transform};
use typed_builder::TypedBuilder;

#[cfg(feature = "server")]
use crate::component::alive::player::SimSync;
#[cfg(feature = "server")]
use bevy::ecs::system::{Commands, EntityCommands};

#[derive(TypedBuilder)]
#[builder(builder_method(vis = ""))]
pub struct Player {
    health: u16,
    mana: u16,
    dps: u16,
    #[builder(default)]
    defense: i16,
    #[builder(default)]
    agility: u8,
    #[builder(default)]
    cdr: u16,
    #[builder(default)]
    luck: u8,
    #[builder(default)]
    init_transform: Transform,
}

#[allow(clippy::type_complexity, reason = "necessary for what i'm up to -_-")]
pub fn player(
    health: u16,
    mana: u16,
    dps: u16,
) -> PlayerBuilder<((u16,), (u16,), (u16,), (), (), (), (), ())> {
    Player::builder().health(health).mana(mana).dps(dps)
}

pub const PLAYER_RADIUS: f64 = 0.5;
pub const PLAYER_HEIGHT: f64 = 2.0;
// -- DON'T CHANGE --
pub const PLAYER_CAPSULE_LENGTH: f64 = PLAYER_HEIGHT - (PLAYER_RADIUS * 2.0);
// -------------------

impl Player {
    pub fn bundle(&self) -> impl Bundle {
        (
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, PLAYER_CAPSULE_LENGTH),
            LockedAxes::ROTATION_LOCKED,
            self.init_transform,
        )
    }

    #[cfg(feature = "server")]
    pub fn spawn<'a>(&self, commands: &'a mut Commands) -> EntityCommands<'a> {
        commands.spawn((
            self.bundle(),
            SimSync::from(self.init_transform),
            Health::new(self.health),
            Mana::new(self.mana),
            Dps::new_complete(self.dps),
            Defense::new_complete(self.defense),
            Agility::new_complete(self.agility),
            Cdr::new_complete(self.cdr),
            Luck::new_complete(self.luck),
        ))
    }
}
