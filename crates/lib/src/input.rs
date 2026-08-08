use bevy::math::Vec2;
use bevy_enhanced_input::prelude::InputAction;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Walk;

#[derive(InputAction)]
#[action_output(bool)]
pub struct Jump;

#[derive(InputAction)]
#[action_output(Vec2)]
pub struct Look;
