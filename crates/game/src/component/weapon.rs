use bevy::{
    asset::Handle,
    audio::{AudioPlayer, AudioSource, PlaybackSettings},
    ecs::{bundle::Bundle, component::Component},
};
use rand::seq::IndexedRandom;

#[derive(Component)]
pub struct WeaponAssets {
    pub sounds: Vec<Handle<AudioSource>>,
}

impl WeaponAssets {
    pub fn sound_bundle(&self) -> impl Bundle {
        (
            #[allow(clippy::unwrap_used, reason = "there's always at least one sound")]
            AudioPlayer(self.sounds.choose(&mut rand::rng()).unwrap().clone()),
            PlaybackSettings::DESPAWN.with_spatial(true),
        )
    }
}
