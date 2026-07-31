use bevy::ecs::resource::Resource;
pub use naia_bevy_shared::Message as Trait;
use naia_bevy_shared::{Message, Serde};

#[derive(Message, Copy, Debug)]
pub enum Input {
    Pressed(Button),
    Released(Button),
}

#[derive(Clone, Copy, Serde, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Button {
    Forward = 1,
    Right = 1 << 1,
    Backward = 1 << 2,
    Left = 1 << 3,
    Jump = 1 << 4,
}

#[derive(Clone, Copy, Default, Resource)]
pub struct Buttons(u8);

bitflags::bitflags! {
    impl Buttons: u8 {
        const FORWARD = Button::Forward as u8;
        const RIGHT = Button::Right as u8;
        const BACKWARD = Button::Backward as u8;
        const LEFT = Button::Left as u8;
        const JUMP = Button::Jump as u8;
    }
}

impl Buttons {
    pub fn apply(&mut self, input: Input) {
        match input {
            Input::Pressed(button) => self.insert(Self(button as u8)),
            Input::Released(button) => self.remove(Self(button as u8)),
        }
    }
}

#[derive(Message)]
pub struct Auth;
