use crate::{
    component::alive::status::{DefenseDown, DefenseUp, DpsDown, DpsUp},
    system::{
        player, spawn,
        status::{poison, stat_change},
        ui::hud::{self, health_bar, mana_bar},
    },
};
use bevy::app::{App, PostStartup, Update};

pub fn movement(app: &mut App) {
    app.add_systems(Update, player::movement)
        .add_observer(player::land)
        .add_observer(player::leave_ground);
}

pub fn status(app: &mut App) {
    app.add_systems(
        Update,
        (
            poison,
            stat_change::<DefenseUp>,
            stat_change::<DefenseDown>,
            stat_change::<DpsUp>,
            stat_change::<DpsDown>,
        ),
    );
}

pub fn hud(app: &mut App) {
    app.add_systems(PostStartup, spawn::hud)
        .add_systems(Update, (health_bar, mana_bar))
        .add_observer(hud::remove_poison);
}
