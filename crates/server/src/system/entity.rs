use crate::Ids;
use avian3d::physics_transform::Position;
use bevy::prelude::*;
use lightyear::prelude::{
    input::bei::{Action, actions},
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
        let client_replicate = Replicate::to_clients(NetworkTarget::Single(id));

        let mut attack = Timer::from_seconds(1.0, TimerMode::Once);
        attack.almost_finish();

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
                actions!(PlayerComponent[
                    (Action::<Walk>::default(), client_replicate.clone()),
                    (Action::<Look>::default(), client_replicate.clone()),
                    (Action::<Jump>::default(), client_replicate.clone()),
                    (Action::<Ability<1>>::default(), client_replicate.clone()),
                    (Action::<Ability<2>>::default(), client_replicate.clone()),
                    (Action::<Ability<3>>::default(), client_replicate.clone()),
                    (Action::<Ability<4>>::default(), client_replicate.clone()),
                    (Action::<Ability<5>>::default(), client_replicate.clone()),
                    (Action::<Attack>::default(), client_replicate.clone()),
                ]),
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
        ))
    }
}
