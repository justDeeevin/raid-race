use crate::{
    component::alive::{Dps, Health, player::AttackTimer},
    event::Attacked,
};
use bevy::{
    app::App,
    ecs::{component::Component, observer::On, system::Query},
};
use lightyear::prelude::AppComponentExt;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize)]
pub struct Warrior {
    pub strike: bool,
    /// Percentage of DPS between 0 and 100
    pub strike_percent: u8,
}

impl Warrior {
    pub fn new(strike_percent: u8) -> Self {
        assert!(
            strike_percent <= 100,
            "strike_percent must be between 0 and 100"
        );
        Self {
            strike: false,
            strike_percent,
        }
    }
}

abilities! {
    Strike(event, mut params| Query<(&mut AttackTimer, &mut Warrior)>) {
        if let Ok((mut timer, mut warrior)) = params.get_mut(**event) {
            tracing::info!("strike!");
            timer.reset();
            warrior.strike = true;
        }
    },
    !Default: [Strike, Strike, Strike, Strike, Strike]
}

fn strike_bonus(
    event: On<Attacked>,
    mut warrior: Query<(&mut Warrior, &Dps)>,
    // mut target: Query<&mut Health>,
) {
    let Ok((mut warrior, Dps(dps))) = warrior.get_mut(event.source) else {
        return;
    };

    if warrior.strike
    // && let Ok(mut target) = target.get_mut(event.target)
    {
        warrior.strike = false;
        // target.current -= (*dps as f32 * (warrior.strike_percent as f32 / 100.0)) as u16
    }
}

pub fn plugin(app: &mut App) {
    add_ability_systems(app);
    app.add_observer(strike_bonus);
    app.component::<Warrior>().replicate();
}
