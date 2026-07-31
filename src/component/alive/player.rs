use bevy::ecs::component::Component;

#[derive(Component, Default)]
#[non_exhaustive]
pub struct PlayerMovable {
    pub ground_contacts: u128,
    pub bhop: bool,
}

#[derive(Component)]
pub struct Player;
