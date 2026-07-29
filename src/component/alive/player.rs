use bevy::ecs::component::Component;

#[derive(Component, Default)]
#[non_exhaustive]
pub struct PlayerMovable {
    pub airborne: bool,
    pub bhop: bool,
}
