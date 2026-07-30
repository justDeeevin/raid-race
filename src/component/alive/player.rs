use bevy::ecs::component::Component;

#[derive(Component)]
#[non_exhaustive]
pub struct PlayerMovable {
    pub airborne: bool,
    pub bhop: bool,
}

impl Default for PlayerMovable {
    fn default() -> Self {
        Self {
            // if spawned grounded, will be set by sensor
            airborne: true,
            bhop: false,
        }
    }
}

#[derive(Component)]
pub struct Player;
