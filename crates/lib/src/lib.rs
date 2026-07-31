pub mod channel;
pub mod component;
pub mod message;
pub mod scene;
pub mod system;

use crate::component::alive::{Agility, Cdr, Defense, Dps, Health, Luck, Mana, player::SimSync};
use message::Auth;
use naia_bevy_shared::{ChannelDirection, ChannelMode, Protocol, TickBufferSettings};
use std::time::Duration;

const TICK_PERIOD: Duration = Duration::from_micros(15625); // 64 Hz
// const TICK_PERIOD: Duration = Duration::from_nanos(7812500); // 128 Hz

pub fn protocol() -> Protocol {
    Protocol::builder()
        .tick_interval(TICK_PERIOD)
        .add_component::<SimSync>()
        .add_component::<Health>()
        .add_component::<Mana>()
        .add_component::<Dps>()
        .add_component::<Agility>()
        .add_component::<Cdr>()
        .add_component::<Defense>()
        .add_component::<Luck>()
        .add_channel::<channel::Input>(
            ChannelDirection::ClientToServer,
            ChannelMode::TickBuffered(TickBufferSettings::default()),
        )
        .add_message::<message::Input>()
        .add_message::<Auth>()
        .build()
}
