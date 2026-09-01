use bevy::prelude::*;
use rand::seq::IndexedRandom;

#[derive(Component)]
pub struct WeaponAssets {
    /// One will be played at random when the player attacks.
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
