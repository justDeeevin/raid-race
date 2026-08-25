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

pub struct Ability<const N: usize>;
pub const N_ABILITIES: usize = 5;

macro_rules! impl_ability {
    ($($n:literal),* $(,)?) => {
        $(
            impl InputAction for Ability<$n> {
                type Output = bool;
            }
        )*
    }
}

impl_ability!(1, 2, 3, 4, 5);

#[derive(InputAction)]
#[action_output(bool)]
pub struct Attack;
