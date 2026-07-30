use crate::component::{
    alive::{Health, Mana},
    ui::{HealthBar, ManaBar},
};
use bevy::{
    ecs::{change_detection::DetectChanges, system::Query, world::Ref},
    ui::{Node, percent, widget::Text},
};

pub fn health_bar(bar: Query<(&mut Node, &HealthBar, &mut Text)>, health: Query<Ref<Health>>) {
    for (mut node, HealthBar(entity), mut text) in bar {
        let health = health.get(*entity).expect("Health bar target not found");
        if health.is_changed() {
            node.width = percent(100.0 * health.current as f32 / health.cap as f32);
            text.0 = format!("{}/{}", health.current, health.cap);
        }
    }
}

pub fn mana_bar(bar: Query<(&mut Node, &ManaBar, &mut Text)>, mana: Query<Ref<Mana>>) {
    for (mut node, ManaBar(entity), mut text) in bar {
        let mana = mana.get(*entity).expect("Mana bar target not found");
        if mana.is_changed() {
            node.width = percent(100.0 * mana.current as f32 / mana.cap as f32);
            text.0 = format!("{}/{}", mana.current, mana.cap);
        }
    }
}
