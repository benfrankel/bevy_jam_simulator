use bevy::prelude::*;
use bevy::window::ExitCondition;
use bevy::window::PresentMode;
use bevy::window::PrimaryWindow;
use bevy::window::WindowMode;
use ron::from_str;
use serde::Deserialize;
use serde::Serialize;
use tap::TapFallible;

use crate::state::editor_screen::EditorScreenConfig;
use crate::state::loading_screen::LoadingScreenConfig;
use crate::state::results_screen::ResultsScreenConfig;
use crate::state::splash_screen::SplashScreenConfig;
use crate::state::title_screen::TitleScreenConfig;
use crate::ui::TooltipConfig;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "web")]
        let config_str = include_str!("../assets/config.ron");
        #[cfg(not(feature = "web"))]
        let config_str = &std::fs::read_to_string("assets/config.ron")
            .tap_err(|e| error!("Reading config: {e}"))
            .unwrap_or_default();
        let config = from_str::<Config>(config_str)
            .tap_err(|e| error!("Deserializing config: {e}"))
            .unwrap();
        info!("Loaded config");

        app.register_type::<Config>()
            .add_plugins(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: config.present_mode,
                    mode: config.window_mode,
                    title: WINDOW_TITLE.to_string(),
                    canvas: Some("#bevy".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                exit_condition: ExitCondition::OnPrimaryClosed,
                ..default()
            })
            .insert_resource(config)
            .add_systems(Update, apply_config.run_if(resource_changed::<Config>()));
    }
}

const WINDOW_TITLE: &str = "bevy_jam4";

// TODO: DevConfig
#[derive(Resource, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct Config {
    // Window
    pub window_mode: WindowMode,
    pub present_mode: PresentMode,
    // TODO: Volume
    // TODO: Mute when out of focus
    // TODO: Keybindings

    // UI
    pub tooltip: TooltipConfig,

    // App states
    pub splash_screen: SplashScreenConfig,
    pub title_screen: TitleScreenConfig,
    pub loading_screen: LoadingScreenConfig,
    pub editor_screen: EditorScreenConfig,
    pub results_screen: ResultsScreenConfig,
}

fn apply_config(config: Res<Config>, mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    info!("Applying config");

    if let Ok(mut window) = window_query.get_single_mut() {
        window.mode = config.window_mode;
        window.present_mode = config.present_mode;
    }

    // TODO: Implement the rest (not important for game jam)
}
