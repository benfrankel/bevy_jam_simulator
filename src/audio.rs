use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use bevy_kira_audio::AudioSource;
use rand::thread_rng;
use rand::Rng;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(KiraAudioPlugin)
            .init_collection::<AudioAssets>();
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/upgrade0.ogg")]
    pub upgrade0: Handle<AudioSource>,
    #[asset(path = "audio/upgrade1.ogg")]
    pub upgrade1: Handle<AudioSource>,
}

impl AudioAssets {
    pub fn random_upgrade(&self) -> Handle<AudioSource> {
        let rng = &mut thread_rng();
        let rand: f32 = rng.gen_range(0.0..1.0);
        if rand < 0.5 {
            self.upgrade0.clone()
        } else {
            self.upgrade1.clone()
        }
    }
}
