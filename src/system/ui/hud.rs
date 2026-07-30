use crate::component::{
    alive::{
        Health, Mana,
        status::{Poison, PoisonRemoved},
    },
    ui::hud::{HealthBar, ManaBar},
};
use bevy::{
    ecs::{change_detection::DetectChanges, observer::On, system::Query, world::Ref},
    ui::{BackgroundColor, Node, percent, widget::Text},
};

pub fn health_bar(
    bar: Query<(&mut Node, &HealthBar, &mut Text, &mut BackgroundColor)>,
    health: Query<(Ref<Health>, Option<Ref<Poison>>)>,
) {
    for (mut node, HealthBar(entity), mut text, mut color) in bar {
        let (health, poison) = health.get(*entity).expect("Health bar target not found");
        if health.is_changed() {
            node.width = percent(100.0 * health.current as f32 / health.cap as f32);
            **text = format!("{}/{}", health.current, health.cap);
        }
        if let Some(poison) = poison
            && poison.is_changed()
        {
            **color = HealthBar::PURPLE;
        }
    }
}

pub fn remove_poison(
    event: On<PoisonRemoved>,
    mut color: Query<(&HealthBar, &mut BackgroundColor)>,
) {
    let Some(mut color) = color.iter_mut().find_map(|(entity, color)| {
        if event.from == entity.0 {
            Some(color)
        } else {
            None
        }
    }) else {
        return;
    };

    **color = HealthBar::RED;
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
