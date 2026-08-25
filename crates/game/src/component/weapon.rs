use bevy::{asset::Handle, audio::AudioSource, ecs::component::Component};

#[derive(Component)]
pub struct WeaponAssets {
    pub sounds: Vec<Handle<AudioSource>>,
}
