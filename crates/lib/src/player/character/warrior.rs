use crate::{
    almost_finish_safe,
    component::alive::{Dps, Health, player::AttackTimer},
    event::{Attacked, Hit},
    player::character::{Abilities, Cooldowns},
};
use bevy::{
    app::{App, FixedUpdate},
    ecs::{
        component::Component,
        observer::On,
        system::{Commands, Query, Res},
    },
    time::{Time, Timer, TimerMode},
};
use either::Either;
use lightyear::prelude::AppComponentExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Component, Serialize, Deserialize)]
pub struct Warrior {
    /// Percentage of DPS between 0 and 100
    pub strike_bonus_percent: u8,
    pub combo_window: Option<Timer>,
    pub strike: bool,
    pub combo: u8,
}

impl Warrior {
    pub fn new(strike_bonus_percent: u8) -> Self {
        assert!(
            strike_bonus_percent <= 100,
            "strike_percent must be between 0 and 100"
        );
        Self {
            strike_bonus_percent,
            combo_window: None,
            strike: false,
            combo: 0,
        }
    }
}

abilities! {
    Strike {
        cast: (event, mut params| Query<(&mut AttackTimer, &mut Warrior, &Abilities<AbilityId>, &mut Cooldowns)>, mut commands| Commands) {
            const COMBO_WINDOW: Duration = Duration::from_secs(3);

            if let Ok((mut timer, mut warrior, abilities, mut cooldowns)) = params.get_mut(**event) {
                almost_finish_safe(&mut timer);
                warrior.combo_window = Some(Timer::new(COMBO_WINDOW, TimerMode::Once));
                warrior.strike = true;
                commands.trigger(Attacked(**event));

                if let Some(slot) = abilities.iter().position(|id| *id == AbilityId::StrikeCombo)
                    && let Some(Either::Right(ready)) = cooldowns.get_mut(slot)
                {
                    *ready = true;
                }
            }
        },
        cooldown: Duration::from_secs(5),
    },
    StrikeCombo {
        cast: (event, mut params| Query<(&mut AttackTimer, &mut Warrior)>, mut commands| Commands) {
            if let Ok((mut timer, mut warrior)) = params.get_mut(**event)
                && let Some(window) = &warrior.combo_window
                && !window.is_finished()
            {
                almost_finish_safe(&mut timer);
                warrior.combo_window = None;
                warrior.combo = 2;
                commands.trigger(Attacked(**event));
            }
        },
        ready: false,
    },
    !Default: [Strike, StrikeCombo, Strike, Strike, Strike]
}

fn strike_bonus(
    event: On<Hit>,
    mut warrior: Query<(&mut Warrior, &Dps)>,
    mut target: Query<&mut Health>,
    mut commands: Commands,
) {
    let Ok((mut warrior, Dps(dps))) = warrior.get_mut(event.source) else {
        return;
    };
    let Ok(mut target) = target.get_mut(event.target) else {
        return;
    };

    if warrior.strike || warrior.combo > 0 {
        target.current = target
            .current
            .saturating_sub((*dps as f32 * (warrior.strike_bonus_percent as f32 / 100.0)) as u16);
    }

    if warrior.strike {
        warrior.strike = false;
    }

    if warrior.combo > 0 {
        warrior.combo -= 1;
        if warrior.combo > 0 {
            commands.trigger(Attacked(event.source));
        }
    }
}

fn combo_window(
    warriors: Query<(&mut Warrior, &mut Cooldowns, &Abilities<AbilityId>)>,
    time: Res<Time>,
) {
    for (mut warrior, mut cooldowns, abilities) in warriors {
        if let Some(timer) = &mut warrior.combo_window {
            timer.tick(time.delta());
            if timer.is_finished()
                && let Some(slot) = abilities
                    .iter()
                    .position(|id| *id == AbilityId::StrikeCombo)
                && let Some(Either::Right(ready)) = cooldowns.get_mut(slot)
            {
                *ready = false;
            }
        }
    }
}

pub fn plugin(app: &mut App) {
    add_ability_systems(app);
    app.add_systems(FixedUpdate, combo_window)
        .add_observer(strike_bonus)
        .component::<Warrior>()
        .replicate();
}
