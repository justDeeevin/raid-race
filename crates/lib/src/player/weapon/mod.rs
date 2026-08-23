pub mod placeholder_gun;

use crate::{
    component::alive::{Dps, Health, player::AttackTimer},
    event::Hit,
};
use bevy::ecs::{observer::On, system::Query};

pub fn hit(event: On<Hit>, damage: Query<(&Dps, &AttackTimer)>, mut health: Query<&mut Health>) {
    let Ok((Dps(damage), AttackTimer(timer))) = damage.get(event.target) else {
        return;
    };

    if let Ok(mut health) = health.get_mut(event.target) {
        health.current = health
            .current
            .saturating_sub((*damage as f32 * timer.duration().as_secs_f32()) as u16);
    }
}
