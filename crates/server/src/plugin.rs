use crate::{
    resource::{Inputs, Room, UserEntities},
    system::{
        player,
        server::{auth, join, leave, serve, tick},
        status::{poison, stat_change},
    },
};
use bevy::{
    app::{App, Startup, Update},
    ecs::schedule::{IntoScheduleConfigs, common_conditions::on_message},
};
use naia_bevy_server::{
    Plugin as NaiaServerPlugin, ServerConfig,
    events::{AuthEvents, ConnectEvent, DisconnectEvent, TickEvent},
};
use raid_race_lib::{
    component::alive::status::{DefenseDown, DefenseUp, DpsDown, DpsUp},
    protocol,
};

pub fn server(app: &mut App) {
    app.add_plugins(NaiaServerPlugin::new(ServerConfig::default(), protocol()))
        .init_resource::<Room>()
        .init_resource::<UserEntities>()
        .add_systems(Startup, serve)
        .add_systems(
            Update,
            (
                join.run_if(on_message::<ConnectEvent>),
                leave.run_if(on_message::<DisconnectEvent>),
                auth.run_if(on_message::<AuthEvents>),
                tick.run_if(on_message::<TickEvent>),
            ),
        );
}

pub fn player(app: &mut App) {
    app.add_observer(player::receive_input)
        .add_observer(player::sync_sim)
        .add_systems(Update, (player::apply_input, player::grounded))
        .init_resource::<Inputs>();
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
