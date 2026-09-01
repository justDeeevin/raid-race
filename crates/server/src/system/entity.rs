use crate::Ids;
use avian3d::physics_transform::Position;
use bevy::prelude::*;
use lightyear::prelude::{
    input::bei::{Action, Actions},
    *,
};
use raid_race_lib::{
    component::alive::{
        Agility, Cdr, Defense, Dps, Health, Luck, Mana,
        player::{AttackCooldown, Pitch, Player as PlayerComponent},
    },
    input::{Ability, Attack, Jump, Look, Walk},
    system::player::{PLAYER_HEIGHT, physics_components},
};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
#[builder(builder_method(vis = ""))]
pub struct Player {
    health: u16,
    mana: u16,
    dps: u16,
    #[builder(default)]
    defense: i16,
    #[builder(default)]
    agility: i16,
    #[builder(default)]
    cdr: u16,
    #[builder(default)]
    luck: u8,
    #[builder(default=Position::from_xyz(0.0, PLAYER_HEIGHT as f64 / 2.0, 0.0))]
    init_pos: Position,
}

pub fn player(
    health: u16,
    mana: u16,
    dps: u16,
) -> PlayerBuilder<((u16,), (u16,), (u16,), (), (), (), (), ())> {
    Player::builder().health(health).mana(mana).dps(dps)
}

fn client_replicate(id: PeerId) -> Replicate {
    Replicate::to_clients(NetworkTarget::Single(id))
}

impl Player {
    pub fn spawn<'a>(
        &self,
        commands: &'a mut Commands,
        id: PeerId,
        ids: &mut Ids,
        owner: Entity,
    ) -> EntityCommands<'a> {
        let mut attack = Timer::from_seconds(1.0, TimerMode::Once);
        attack.finish();

        let mut out = commands.spawn((
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
                AttackCooldown(attack),
            ),
            physics_components(),
            self.init_pos,
            ControlledBy {
                owner,
                lifetime: Lifetime::SessionBased,
            },
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(id)),
        ));

        out.apply_scene(bsn! {
            Actions<PlayerComponent> [
                Action::<Walk>
                client_replicate(id)
                --
                Action::<Look>
                client_replicate(id)
                --
                Action::<Jump>
                client_replicate(id)
                --
                Action::<Ability<1>>
                client_replicate(id)
                --
                Action::<Ability<2>>
                client_replicate(id)
                --
                Action::<Ability<3>>
                client_replicate(id)
                --
                Action::<Ability<4>>
                client_replicate(id)
                --
                Action::<Ability<5>>
                client_replicate(id)
                --
                Action::<Attack>
                client_replicate(id)
            ]
        });

        out
    }
}
