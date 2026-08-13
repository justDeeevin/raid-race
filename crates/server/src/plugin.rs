use crate::system::{
    player,
    server::{self, Ids},
    status,
};
use bevy::app::{App, FixedUpdate, Startup, Update};
use lightyear::prelude::server::ServerPlugins;
use raid_race_lib::{
    TICK_PERIOD,
    component::alive::status::{DefenseDown, DefenseUp, DpsDown, DpsUp},
};

pub fn server(app: &mut App) {
    app.add_plugins((
        ServerPlugins {
            tick_duration: TICK_PERIOD,
        },
        raid_race_lib::plugin,
    ))
    .add_systems(Startup, server::serve)
    .add_observer(server::start_join)
    .add_observer(server::join)
    .add_observer(server::leave)
    .init_resource::<Ids>();
}

pub fn player(app: &mut App) {
    app.add_plugins(raid_race_lib::player::plugin)
        .add_systems(FixedUpdate, player::grounded)
        .add_observer(player::landed)
        .add_observer(player::jump_released)
        .add_observer(player::leave_ground);
}

pub fn status(app: &mut App) {
    app.add_systems(
        Update,
        (
            status::poison,
            status::stat_change::<DefenseUp>,
            status::stat_change::<DefenseDown>,
            status::stat_change::<DpsUp>,
            status::stat_change::<DpsDown>,
        ),
    );
}
