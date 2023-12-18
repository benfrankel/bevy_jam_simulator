use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(KiraAudioPlugin)
            .init_collection::<AudioAssets>();

        // Music will be played by Web Audio API on web.
        #[cfg(not(feature = "web"))]
        app.register_type::<BackgroundMusic>()
            .init_resource::<BackgroundMusic>()
            .add_systems(Startup, spawn_background_music);
    }
}

#[derive(Clone, Copy)]
pub enum SoundEffectKind {
    DefaultUpgrade,
    Keyboard,
    Backspace,
    Guitar,
    Unicorn,
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct AudioAssets {
    #[asset(paths("audio/upgrade0.ogg", "audio/upgrade1.ogg"), collection(typed))]
    pub upgrade_sounds: Vec<Handle<AudioSource>>,
    #[asset(
        paths("audio/keyboard0.ogg", "audio/keyboard1.ogg", "audio/keyboard2.ogg"),
        collection(typed)
    )]
    pub keyboard_sounds: Vec<Handle<AudioSource>>,
    #[asset(path = "audio/backspace0.ogg")]
    pub backspace_sound: Handle<AudioSource>,
    #[asset(
        paths("audio/guitar0.ogg", "audio/guitar1.ogg", "audio/guitar2.ogg"),
        collection(typed)
    )]
    pub guitar_sounds: Vec<Handle<AudioSource>>,
    #[asset(paths("audio/unicorn0.ogg", "audio/unicorn1.ogg"), collection(typed))]
    pub unicorn_sounds: Vec<Handle<AudioSource>>,

    #[cfg(not(feature = "web"))]
    #[asset(path = "music/ingame.ogg")]
    pub music: Handle<AudioSource>,
}

impl AudioAssets {
    pub fn get_sfx(&self, kind: SoundEffectKind) -> Handle<AudioSource> {
        use SoundEffectKind::*;
        match kind {
            DefaultUpgrade => &self.upgrade_sounds,
            Keyboard => &self.keyboard_sounds,
            Backspace => return self.backspace_sound.clone(),
            Guitar => &self.guitar_sounds,
            Unicorn => &self.unicorn_sounds,
        }
        .choose(&mut thread_rng())
        .unwrap()
        .clone()
    }
}

#[cfg(not(feature = "web"))]
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct BackgroundMusic(pub Handle<AudioInstance>);

#[cfg(not(feature = "web"))]
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
