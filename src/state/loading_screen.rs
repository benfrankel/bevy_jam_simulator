use bevy::core::FrameCount;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::state::editor_screen::EditorScreenAssets;
use crate::state::AppState::*;
use crate::ui::vmin;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;
use crate::AppRoot;

pub struct LoadingScreenStatePlugin;

impl Plugin for LoadingScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsLoadingBarFill>()
            .add_loading_state(LoadingState::new(LoadingScreen))
            .add_collection_to_loading_state::<_, EditorScreenAssets>(LoadingScreen)
            .add_plugins(ProgressPlugin::new(LoadingScreen).continue_to(EditorScreen))
            .add_systems(OnEnter(LoadingScreen), enter_loading)
            .add_systems(OnExit(LoadingScreen), exit_loading)
            .add_systems(
                Update,
                update_loading
                    .run_if(in_state(LoadingScreen))
                    .after(TrackedProgressSet),
            );
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct LoadingScreenConfig {
    foreground_color: Color,
    background_color: Color,
    border_color: Color,
    border_width: Val,
    font_size: Val,
}

#[derive(Component, Reflect)]
struct IsLoadingBarFill;

fn enter_loading(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    let config = &config.loading_screen;
    commands.insert_resource(ClearColor(config.background_color));

    let screen = commands
        .spawn((
            Name::new("LoadingScreen"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    commands
        .spawn((
            Name::new("LoadingContainer"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(screen)
        .with_children(|commands| {
            commands.spawn((
                Name::new("LoadingHeader"),
                TextBundle {
                    style: Style {
                        margin: UiRect::all(vmin(8.0)),
                        ..default()
                    },
                    text: Text::from_section(
                        "Loading...",
                        TextStyle {
                            font: BOLD_FONT_HANDLE,
                            color: config.foreground_color,
                            ..default()
                        },
                    ),
                    ..default()
                },
                FontSize::new(config.font_size),
            ));

            commands
                .spawn((
                    Name::new("LoadingBarContainer"),
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(60.0),
                            height: Val::Percent(7.5),
                            border: UiRect::all(config.border_width),
                            ..default()
                        },
                        border_color: config.border_color.into(),
                        ..default()
                    },
                ))
                .with_children(|commands| {
                    commands.spawn((
                        Name::new("LoadingBarFill"),
                        NodeBundle {
                            background_color: BackgroundColor(config.foreground_color),
                            style: Style {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            ..default()
                        },
                        IsLoadingBarFill,
                    ));
                });
        });
}

fn exit_loading(mut commands: Commands, root: Res<AppRoot>) {
    commands.entity(root.ui).despawn_descendants();
}

fn update_loading(
    mut loading_bar_query: Query<&mut Style, With<IsLoadingBarFill>>,
    progress: Res<ProgressCounter>,
    frame: Res<FrameCount>,
    mut last_done: Local<u32>,
) {
    let Progress { done, total } = progress.progress();
    if *last_done == done {
        return;
    }
    *last_done = done;

    for mut style in &mut loading_bar_query {
        style.width = Val::Percent(100.0 * done as f32 / total as f32);
    }

    info!("[Frame {}] Loading: {done} / {total}", frame.0);
}
