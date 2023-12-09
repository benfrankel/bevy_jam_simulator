use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use rand::thread_rng;
use rand::Rng;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<BackgroundMusic>()
            .add_plugins(KiraAudioPlugin)
            .init_collection::<AudioAssets>()
            .init_resource::<BackgroundMusic>()
            .add_systems(Startup, spawn_background_music);
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/upgrade0.ogg")]
    pub upgrade0: Handle<AudioSource>,
    #[asset(path = "audio/upgrade1.ogg")]
    pub upgrade1: Handle<AudioSource>,
    #[asset(path = "music/ingame.ogg")]
    pub music: Handle<AudioSource>,
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

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct BackgroundMusic(pub Handle<AudioInstance>);

fn spawn_background_music(
    mut commands: Commands,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    let handle = audio
        .play(audio_assets.music.clone())
        .with_volume(0.8)
        .looped()
        .paused()
        .handle();
    commands.insert_resource(BackgroundMusic(handle));
}
