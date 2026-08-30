use crate::{
    component::alive::{
        Agility, Dps, Health,
        player::{
            AttackCooldown,
            character::{Character, CharacterData, Cooldowns},
        },
        status::{AgilityDown, StackableStatusEffect},
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
        cast: (event, mut params| Query<(&mut AttackCooldown, &mut Character, &mut Cooldowns)>, mut commands| Commands) {
            const COMBO_WINDOW: Duration = Duration::from_secs(3);

            if let Ok((mut timer, mut character, mut cooldowns)) = params.get_mut(**event)
                && let CharacterData::Warrior {
                    strike,
                    combo_window,
                    combo_slot,
                    ..
                } = &mut character.data
            {
                timer.finish();
                *combo_window = Some(Timer::new(COMBO_WINDOW, TimerMode::Once));
                *strike = true;
                commands.trigger(Attacked(**event));

                if let Some(slot) = combo_slot
                    && let Some(Either::Right(ready)) = cooldowns.get_mut(*slot - 1)
                {
                    *ready = true;
                }
            }
        },
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
        ready: false,
    },
    Spin {
        cast: (event, mut characters| Query<&mut Character>) {
            const SPIN_DURATION: Duration = Duration::from_secs(1);

            if let Ok(mut character) = characters.get_mut(**event)
                && let CharacterData::Warrior { spin, .. } = &mut character.data
            {
                let mut timer = Timer::new(SPIN_DURATION / 5, TimerMode::Repeating);
                timer.finish();
                *spin = Some((0, timer));
            }
        },
        cooldown: Duration::from_secs(10),
    },
    Meditate {
        cast: (event, mut characters| Query<&mut Character>) {
            const CHANNEL_DURATION: Duration = Duration::from_millis(2500);
            const N_TICKS: u32 = 5;

            if let Ok(mut character) = characters.get_mut(**event)
            {
                let prev = character.channel.take();

                character.channel = Some(Timer::new(CHANNEL_DURATION, TimerMode::Once));

                if let CharacterData::Warrior { meditate, .. } = &mut character.data {
                    *meditate = Some(Timer::new(CHANNEL_DURATION / N_TICKS, TimerMode::Repeating))
                } else {
                    character.channel = prev;
                }
            }
        },
        cooldown: Duration::from_secs(10),
    },
    Kick {
        cast: (event, space| SpatialQuery, params| Query<(&Position, &Rotation)>, mut targets| Query<&mut Health, With<Agility>>, mut commands| Commands) {
            const HITBOX_DIMENSIONS: Vector = Vector::new(0.5, 0.1, 0.25);
            #[allow(clippy::unwrap_used, reason = "this is const")]
            const STACKS: NonZero<u8> = NonZero::new(40).unwrap();
            const DURATION: Duration = Duration::from_secs(5);
            const DAMAGE: u16 = 6;

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
                    if let Ok(mut health) = targets.get_mut(hit) {
                        commands.entity(hit).insert(AgilityDown(StackableStatusEffect::new(STACKS, DURATION)));
                        *health -= DAMAGE;
                    }
                }
            }
        },
        cooldown: Duration::from_secs(10),
    }
}

fn strike_bonus(
    event: On<Hit>,
    mut characters: Query<(&mut Character, &Dps)>,
    mut target: Query<&mut Health>,
    mut commands: Commands,
) {
    if let Ok((mut character, Dps(dps))) = characters.get_mut(event.source)
        && let CharacterData::Warrior {
            strike,
            combo,
            strike_bonus_percent,
            ..
        } = &mut character.data
        && let Ok(mut target) = target.get_mut(event.target)
    {
        if *strike || *combo > 0 {
            *target -= (*dps as f32 * (*strike_bonus_percent as f32 / 100.0)) as u16;
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
            combo_slot,
            ..
        } = &mut character.data
            && let Some(timer) = combo_window
            && timer.tick(time.delta()).is_finished()
            && let Some(slot) = combo_slot
            && let Some(Either::Right(ready)) = cooldowns.get_mut(*slot - 1)
        {
            *ready = false;
        }
    }
}

fn spin(
    characters: Query<(Entity, &mut Character, &Position, &Dps)>,
    mut healths: Query<&mut Health>,
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

        if let Some((n, timer)) = spin_timer {
            if timer.just_finished() {
                *n += 1;
                for hit in space.shape_intersections(
                    &Collider::cylinder(AOE_RADIUS, AOE_HEIGHT),
                    **position,
                    Quaternion::default(),
                    &SpatialQueryFilter::from_excluded_entities([entity]),
                ) {
                    if let Ok(mut health) = healths.get_mut(hit) {
                        *health -= (**dps as f32
                            * timer.duration().as_secs_f32()
                            * DAMAGE_MULTIPLIER) as u16;
                    }
                }
            }

            timer.tick(time.delta());
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

pub fn plugin(app: &mut App) {
    add_ability_systems(app);
    app.add_systems(FixedUpdate, (combo_window, spin, meditate))
        .add_observer(strike_bonus);
}
