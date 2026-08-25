use bevy::{asset::Handle, audio::AudioSource, ecs::component::Component};

#[derive(Component)]
pub struct PlaceholderGunAssets {
    pub sounds: Vec<Handle<AudioSource>>,
}
