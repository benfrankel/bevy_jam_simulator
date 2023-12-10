use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_kira_audio::AudioPlugin as KiraAudioPlugin;
use rand::seq::SliceRandom;
use rand::thread_rng;

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
    #[asset(path = "music/ingame.ogg")]
    pub music: Handle<AudioSource>,
}

impl AudioAssets {
    pub fn get_sfx(&self, kind: SoundEffectKind) -> Handle<AudioSource> {
        use SoundEffectKind::*;
        macro_rules! select_from {
            ($a:expr) => {
                $a.choose(&mut thread_rng()).unwrap().clone()
            };
        }
        match kind {
            DefaultUpgrade => select_from!(self.upgrade_sounds),
            Keyboard => select_from!(self.keyboard_sounds),
            Backspace => self.backspace_sound.clone(),
            Guitar => select_from!(self.guitar_sounds),
            Unicorn => select_from!(self.unicorn_sounds),
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
