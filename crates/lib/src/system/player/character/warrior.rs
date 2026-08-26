use crate::{
    component::alive::{
        Dps, Health,
        player::{
            AttackCooldown,
            character::{Character, Cooldowns},
        },
    },
    event::{Attacked, Hit},
    system::player::PLAYER_HEIGHT,
};
use avian3d::{
    collision::collider::Collider,
    math::Quaternion,
    physics_transform::Position,
    spatial_query::{SpatialQuery, SpatialQueryFilter},
};
use bevy::{
    app::{App, FixedUpdate},
    ecs::{
        entity::Entity,
        observer::On,
        system::{Commands, Query, Res},
    },
    time::{Time, Timer, TimerMode},
};
use either::Either;
use std::time::Duration;

abilities!(warrior {
    Strike {
        cast: (event, mut params| Query<(&mut AttackCooldown, &mut Character, &mut Cooldowns)>, mut commands| Commands) {
            const COMBO_WINDOW: Duration = Duration::from_secs(3);

            if let Ok((mut timer, mut character, mut cooldowns)) = params.get_mut(**event)
                && let Character::Warrior {strike, combo_window, combo_slot, ..} = character.as_mut()
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
                && let Character::Warrior {combo_window, combo, ..} = character.as_mut()
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

            if let Ok(mut character) = characters.get_mut(**event) && let Character::Warrior {spin_timer, ..} = character.as_mut() {
                let mut timer = Timer::new(SPIN_DURATION / 5, TimerMode::Repeating);
                timer.finish();
                *spin_timer = Some((0, timer));
            }
        },
        cooldown: Duration::from_secs(10),
    },
});

fn strike_bonus(
    event: On<Hit>,
    mut characters: Query<(&mut Character, &Dps)>,
    mut target: Query<&mut Health>,
    mut commands: Commands,
) {
    if let Ok((mut character, Dps(dps))) = characters.get_mut(event.source)
        && let Character::Warrior {
            strike,
            combo,
            strike_bonus_percent,
            ..
        } = character.as_mut()
        && let Ok(mut target) = target.get_mut(event.target)
    {
        if *strike || *combo > 0 {
            target.current = target
                .current
                .saturating_sub((*dps as f32 * (*strike_bonus_percent as f32 / 100.0)) as u16);
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
        if let Character::Warrior {
            combo_window,
            combo_slot,
            ..
        } = character.as_mut()
            && let Some(timer) = combo_window
        {
            timer.tick(time.delta());
            if timer.is_finished()
                && let Some(slot) = combo_slot
                && let Some(Either::Right(ready)) = cooldowns.get_mut(*slot - 1)
            {
                *ready = false;
            }
        }
    }
}

fn spin(
    characters: Query<(Entity, &mut Character, &Position, &Dps)>,
    mut healths: Query<&mut Health>,
    space: SpatialQuery,
    time: Res<Time>,
) {
    const AOE_RADIUS: f64 = 1.0;
    const AOE_HEIGHT: f64 = PLAYER_HEIGHT;
    const DAMAGE_MULTIPLIER: f32 = 2.0;

    for (entity, mut character, position, dps) in characters {
        let Character::Warrior { spin_timer, .. } = character.as_mut() else {
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
                        health.current = health.current.saturating_sub(
                            (**dps as f32 * timer.duration().as_secs_f32() * DAMAGE_MULTIPLIER)
                                as u16,
                        );
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

pub fn plugin(app: &mut App) {
    add_ability_systems(app);
    app.add_systems(FixedUpdate, (combo_window, spin))
        .add_observer(strike_bonus);
}
