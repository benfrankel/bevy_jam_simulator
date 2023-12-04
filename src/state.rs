pub mod editor_screen;
pub mod results_screen;
pub mod loading_screen;
pub mod splash_screen;
pub mod title_screen;

use bevy::prelude::*;
use strum::EnumIter;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>().add_plugins((
            splash_screen::SplashScreenStatePlugin,
            title_screen::TitleScreenStatePlugin,
            loading_screen::LoadingScreenStatePlugin,
            editor_screen::EditorScreenStatePlugin,
            results_screen::ResultsScreenStatePlugin,
        ));
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(States, Default, Copy, Clone, Eq, PartialEq, Hash, Debug, EnumIter)]
pub enum AppState {
    #[default]
    SplashScreen,
    TitleScreen,
    LoadingScreen,
    EditorScreen,
    ResultsScreen,
}
