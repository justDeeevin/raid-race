use avian3d::{
    collision::collider::{Collider, Sensor},
    physics_transform::Position,
};
use bevy::ecs::{
    children,
    entity::Entity,
    system::{Commands, EntityCommands},
};
use lightyear::{
    connection::network_target::NetworkTarget,
    core::id::PeerId,
    prelude::{
        ControlledBy, Lifetime, PredictionTarget, Replicate,
        input::bei::{Action, actions},
    },
};
use raid_race_lib::{
    component::alive::{
        Agility, Cdr, Defense, Dps, Health, Luck, Mana,
        player::{CanJump, Pitch, Player as PlayerComponent},
    },
    input::{Jump, Look, Walk},
    player::{PLAYER_HEIGHT, PLAYER_RADIUS, physics_components},
};
use typed_builder::TypedBuilder;

use crate::Ids;

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
    #[builder(default=Position::from_xyz(0.0, PLAYER_HEIGHT / 2.0, 0.0))]
    init_pos: Position,
}

pub fn player(
    health: u16,
    mana: u16,
    dps: u16,
) -> PlayerBuilder<((u16,), (u16,), (u16,), (), (), (), (), ())> {
    Player::builder().health(health).mana(mana).dps(dps)
}

impl Player {
    pub fn spawn<'a>(
        &self,
        commands: &'a mut Commands,
        id: PeerId,
        ids: &mut Ids,
        owner: Entity,
    ) -> EntityCommands<'a> {
        const FOOT_HEIGHT: f64 = 0.02;

        let client_replicate = Replicate::to_clients(NetworkTarget::Single(id));

        commands.spawn((
            (
                PlayerComponent(id),
                ids.get(),
                Health::new(self.health),
                Mana::new(self.mana),
                Dps(self.dps),
                Defense(self.defense),
                Agility(self.agility),
                Cdr(self.cdr),
                Luck(self.luck),
                Pitch(0.0),
                actions!(PlayerComponent[(Action::<Walk>::new(), client_replicate.clone()), (Action::<Look>::new(), client_replicate.clone())]),
                actions!(CanJump[(Action::<Jump>::new(), client_replicate.clone())]),
            ),
            physics_components(),
            self.init_pos,
            ControlledBy {
                owner,
                lifetime: Lifetime::SessionBased,
            },
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(id)),
            children![(
                Collider::cylinder(PLAYER_RADIUS, FOOT_HEIGHT),
                Sensor,
                Position::from_xyz(0.0, (-PLAYER_HEIGHT - FOOT_HEIGHT) / 2.0, 0.0),
            )],
        ))
    }
}
