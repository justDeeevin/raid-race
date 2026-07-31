use bevy::ecs::component::Component;

#[derive(Component, Default)]
pub struct Movement {
    pub ground_contacts: usize,
    pub bhop: bool,
}
