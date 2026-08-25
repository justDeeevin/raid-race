use crate::{
    almost_finish_safe,
    component::alive::{
        Dps, Health,
        player::{
            AttackCooldown,
            character::{Character, Cooldowns},
        },
    },
    event::{Attacked, Hit},
};
use bevy::{
    app::{App, FixedUpdate},
    ecs::{
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
                almost_finish_safe(&mut timer);
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
                almost_finish_safe(&mut timer);
                *combo_window = None;
                *combo = 2;
                commands.trigger(Attacked(**event));
            }
        },
        ready: false,
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
            if *combo > 0 {
                commands.trigger(Attacked(event.source));
            }
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

pub fn plugin(app: &mut App) {
    add_ability_systems(app);
    app.add_systems(FixedUpdate, combo_window)
        .add_observer(strike_bonus);
}
