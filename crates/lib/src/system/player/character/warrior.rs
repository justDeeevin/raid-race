use crate::{
    component::alive::{
        Agility, Defense, Dps, Health,
        player::{
            AttackCooldown,
            character::{Character, CharacterData, Cooldowns},
        },
        status::{AgilityDown, DefenseDown, DpsUp, StackableStatusEffect},
    },
    event::{Attacked, Hit},
    system::player::{PLAYER_HEIGHT, PLAYER_RADIUS},
};
use avian3d::{
    math::{Quaternion, Scalar, Vector},
    prelude::*,
};
use bevy::prelude::*;
use either::Either;
use std::{num::NonZero, time::Duration};

// TODO: LOS checks?
abilities! {
    Strike {
        cast: (
            event,
            mut params| Query<(&mut AttackCooldown, &mut Character, &mut Cooldowns)>,
            mut commands| Commands
        ) {
            const COMBO_WINDOW: Duration = Duration::from_secs(3);

            if let Ok((mut timer, mut character, mut cooldowns)) = params.get_mut(**event)
                && let CharacterData::Warrior {
                    strike,
                    combo_window,
                    combo_index,
                    ..
                } = &mut character.data
            {
                timer.finish();
                *combo_window = Some(Timer::new(COMBO_WINDOW, TimerMode::Once));
                *strike = true;
                commands.trigger(Attacked(**event));

                if let Some(slot) = combo_index
                    && let Some(Either::Right(ready)) = cooldowns.get_mut(*slot - 1)
                {
                    *ready = true;
                }
            }
        },
        description: "Attack with additional damage and reset attack cooldown.",
        cooldown: Duration::from_secs(5),
    },
    StrikeCombo {
        cast: (event, mut params| Query<(&mut AttackCooldown, &mut Character)>, mut commands| Commands) {
            if let Ok((mut timer, mut character)) = params.get_mut(**event)
                && let CharacterData::Warrior { combo_window, combo, .. } = &mut character.data
                && let Some(window) = &combo_window
                && !window.is_finished()
            {
                timer.finish();
                *combo_window = None;
                *combo = 2;
                commands.trigger(Attacked(**event));
            }
        },
        description: "Can be cast within a short window of casting Strike.\nAttack twice, each with additional strike damage, and reset attack cooldown.",
        name: "Strike Combo",
    },
    // TODO: should channel?
    Spin {
        cast: (event, mut characters| Query<&mut Character>) {
            const DURATION: Duration = Duration::from_secs(1);

            if let Ok(mut character) = characters.get_mut(**event)
                && let CharacterData::Warrior { spin, .. } = &mut character.data
            {
                let mut timer = Timer::new(DURATION / 5, TimerMode::Repeating);
                timer.almost_finish();
                *spin = Some((0, timer));
            }
        },
        description: "Spin in place, dealing damage to nearby enemies.",
        cooldown: Duration::from_secs(10),
    },
    Meditate {
        cast: (event, mut characters| Query<&mut Character>) {
            const DURATION: Duration = Duration::from_millis(2500);
            const N_TICKS: u32 = 5;

            if let Ok(mut character) = characters.get_mut(**event)
            {
                let prev = character.channel.take();

                character.channel = Some(Timer::new(DURATION, TimerMode::Once));

                if let CharacterData::Warrior { meditate, .. } = &mut character.data {
                    *meditate = Some(Timer::new(DURATION / N_TICKS, TimerMode::Repeating))
                } else {
                    character.channel = prev;
                }
            }
        },
        description: "Channel to heal a fraction of your maximum health.",
        cooldown: Duration::from_secs(10),
    },
    Kick {
        cast: (
            event,
            space| SpatialQuery,
            params| Query<(&Position, &Rotation)>,
            mut targets| Query<(&mut Health, &Defense), With<Agility>>,
            mut commands| Commands
        ) {
            #[allow(clippy::unwrap_used, reason = "const")]
            const AGILITY_DOWN_STACKS: NonZero<u8> = NonZero::new(40).unwrap();
            const DEBUFF_DURATION: Duration = Duration::from_secs(5);
            const DAMAGE: u16 = 6;
            const HITBOX_DIMENSIONS: Vector = Vector::new(0.5, 0.1, 0.25);

            if let Ok((position, rotation)) = params.get(**event) {
                for hit in space.shape_intersections(
                    &Collider::cuboid(HITBOX_DIMENSIONS.x, HITBOX_DIMENSIONS.y, HITBOX_DIMENSIONS.z),
                    **position + (**rotation * Vector::new(
                        0.0,
                        (-PLAYER_HEIGHT / 2.0) + (HITBOX_DIMENSIONS.y / 2.0),
                        -PLAYER_RADIUS - (HITBOX_DIMENSIONS.z / 2.0),
                    )),
                    **rotation,
                    &default(),
                ) {
                    if let Ok((mut health, defense)) = targets.get_mut(hit) {
                        commands.entity(hit)
                            .insert(AgilityDown(StackableStatusEffect::new(AGILITY_DOWN_STACKS, DEBUFF_DURATION)));
                        health.damage(DAMAGE, **defense);
                    }
                }
            }
        },
        description: "Swipe an area in front of your feet, damaging and decreasing the agility of those hit.",
        cooldown: Duration::from_secs(10),
    },
    Leap {
        cast: (event, mut params| Query<(&Rotation, &mut LinearVelocity)>) {
            if let Ok((rotation, mut velocity)) = params.get_mut(**event) {
                **velocity += **rotation * Vector::new(0.0, 6.0, -8.0);
            }
        },
        description: "Leap into the air.",
        cooldown: Duration::from_secs(10),
    },
    Trance {
        cast: (
            event,
            mut params| Query<(&Dps, &mut AttackCooldown, &mut Character)>,
            mut commands| Commands
        ) {
            const DURATION: Duration = Duration::from_secs(7);

            if let Ok((Dps(dps), mut timer, mut character)) = params.get_mut(**event)
                && let CharacterData::Warrior {trance, ..} = &mut character.data
            {
                let new_cd = timer.duration() - (timer.duration() / 4);
                timer.set_duration(new_cd);

                *trance = Some(Timer::new(DURATION, TimerMode::Once));

                if let Some(stacks) = NonZero::new((*dps as f32 * 1.25) as u8) {
                    commands.entity(**event)
                        .insert(DpsUp(StackableStatusEffect::new(stacks, DURATION)));
                }

            }
        },
        description: "Enter a trance state, increasing your damage and decreasing your attack cooldown. You cannot fall below 5% of your maximum health while entranced.",
        name: "Battle Trance",
        cooldown: Duration::from_secs(10),
    },
    Crimp {
        cast: (
            event,
            sources| Query<(&Position, &Rotation)>,
            targets| Query<&Defense>,
            space| SpatialQuery,
            mut commands| Commands
        ) {
            const DEBUFF_DURATION: Duration = Duration::from_secs(2);

            const START_WIDTH: Scalar = 1.0;
            const END_WIDTH: Scalar = 1.0;
            const LENGTH: Scalar = 1.0;
            const HEIGHT: Scalar= 1.0;

            #[allow(clippy::unwrap_used, reason = "static definition")]
            let collider = Collider::convex_hull(vec![
                Vector::new(-START_WIDTH / 2.0, 0.0, 0.0),
                Vector::new(-END_WIDTH / 2.0, 0.0, -LENGTH),
                Vector::new(END_WIDTH / 2.0, 0.0, -LENGTH),
                Vector::new(START_WIDTH / 2.0, 0.0, 0.0),
                Vector::new(-START_WIDTH / 2.0, HEIGHT, 0.0),
                Vector::new(-END_WIDTH / 2.0, HEIGHT, -LENGTH),
                Vector::new(END_WIDTH / 2.0, HEIGHT, -LENGTH),
                Vector::new(START_WIDTH / 2.0, HEIGHT, 0.0),
            ]).unwrap();

            if let Ok((position, rotation)) = sources.get(**event) {
                commands.spawn((
                    collider.clone(),
                    Position(**position + (**rotation * Vector::new(0.0, 0.0, -PLAYER_RADIUS))),
                    *rotation,
                ));

                for hit in space.shape_intersections(
                    &collider,
                    **position + (**rotation * Vector::new(0.0, -HEIGHT / 2.0, -PLAYER_RADIUS)),
                    **rotation,
                    &Default::default()
                ) {
                    if let Ok(defense) = targets.get(hit)
                    {
                        #[allow(clippy::unwrap_used, reason = "cannot panic")]
                        commands.entity(hit)
                            .insert(DefenseDown(StackableStatusEffect::new(
                                NonZero::new(((**defense as f32 * 0.15) as u8).max(1)).unwrap(),
                                DEBUFF_DURATION
                            )));
                    }
                }
            }
        },
        description: "Temporarily decrease the defense of enemies in a cone in front of you.",
        cooldown: Duration::from_secs(10),
    },
}

fn strike_bonus(
    event: On<Hit>,
    mut characters: Query<(&mut Character, &Dps)>,
    mut targets: Query<(&mut Health, &Defense)>,
    mut commands: Commands,
) {
    if let Ok((mut character, Dps(dps))) = characters.get_mut(event.source)
        && let CharacterData::Warrior {
            strike,
            combo,
            strike_bonus_percent,
            ..
        } = &mut character.data
        && let Ok((mut health, defense)) = targets.get_mut(event.target)
    {
        if *strike || *combo > 0 {
            health.damage(
                (*dps as f32 * (*strike_bonus_percent as f32 / 100.0)) as u16,
                **defense,
            );
        }

        if *strike {
            *strike = false;
        }

        if *combo > 0 {
            *combo -= 1;
            commands.trigger(Attacked(event.source));
        }
    }
}

fn combo_window(warriors: Query<(&mut Character, &mut Cooldowns)>, time: Res<Time>) {
    for (mut character, mut cooldowns) in warriors {
        if let CharacterData::Warrior {
            combo_window,
            combo_index,
            ..
        } = &mut character.data
            && let Some(timer) = combo_window
            && timer.tick(time.delta()).is_finished()
            && let Some(slot) = combo_index
            && let Some(Either::Right(ready)) = cooldowns.get_mut(*slot - 1)
        {
            *ready = false;
        }
    }
}

fn spin(
    characters: Query<(Entity, &mut Character, &Position, &Dps)>,
    mut healths: Query<(&mut Health, &Defense)>,
    space: SpatialQuery,
    time: Res<Time>,
) {
    const AOE_RADIUS: Scalar = 3.0;
    const AOE_HEIGHT: Scalar = PLAYER_HEIGHT;
    const DAMAGE_MULTIPLIER: f32 = 2.0;

    for (entity, mut character, position, dps) in characters {
        let CharacterData::Warrior {
            spin: spin_timer, ..
        } = &mut character.data
        else {
            continue;
        };

        if let Some((n, timer)) = spin_timer
            && timer.tick(time.delta()).just_finished()
        {
            *n += 1;
            for hit in space.shape_intersections(
                &Collider::cylinder(AOE_RADIUS, AOE_HEIGHT),
                **position,
                Quaternion::default(),
                &SpatialQueryFilter::from_excluded_entities([entity]),
            ) {
                if let Ok((mut health, defense)) = healths.get_mut(hit) {
                    health.damage(
                        (**dps as f32 * timer.duration().as_secs_f32() * DAMAGE_MULTIPLIER) as u16,
                        **defense,
                    );
                }
            }
        }

        if spin_timer.as_ref().is_some_and(|(n, _)| *n >= 5) {
            *spin_timer = None;
        }
    }
}

fn meditate(characters: Query<(&mut Character, &mut Health)>, time: Res<Time>) {
    const HEAL_PERCENT_PER_TICK: u8 = 2;

    for (mut character, mut health) in characters {
        let finished = character.channel.as_ref().is_none_or(Timer::is_finished);

        if let CharacterData::Warrior { meditate, .. } = &mut character.data {
            if finished && meditate.is_some() {
                *meditate = None;
            } else if let Some(timer) = meditate
                && timer.tick(time.delta()).just_finished()
            {
                let cap = health.cap as f32;
                *health += (cap * (HEAL_PERCENT_PER_TICK as f32 * 0.01)) as u16;
            }
        }
    }
}

fn trance(characters: Query<(&mut Character, &mut Health, &mut AttackCooldown)>, time: Res<Time>) {
    for (mut character, mut health, mut cd) in characters {
        if let CharacterData::Warrior { trance, .. } = &mut character.data
            && let Some(timer) = trance
        {
            if timer.tick(time.delta()).just_finished() {
                *trance = None;

                let new_cd = cd.duration() + cd.duration() / 3;
                cd.set_duration(new_cd);
            } else {
                health.current = health.current.max((health.cap as f32 * 0.05) as u16);
            }
        }
    }

    pub type Thing = ();
}

pub fn plugin(app: &mut App) {
    add_ability_systems(app);
    app.add_systems(FixedUpdate, (combo_window, spin, meditate, trance))
        .add_observer(strike_bonus);
}
