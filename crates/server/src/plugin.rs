use crate::system::{
    console,
    server::{self, ClientIds},
    status,
};
use bevy::{
    app::{App, Startup, Update},
    platform::cell::SyncCell,
};
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
    .init_resource::<ClientIds>();
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

pub fn console(app: &mut App) {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(console::thread(tx));

    console::handle(SyncCell::new(rx), app);

    app.add_observer(console::poison)
        .add_observer(console::slot)
        .add_observer(console::character);
}
