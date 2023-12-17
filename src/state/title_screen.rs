use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::prelude::*;
use iyes_progress::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::audio::AudioAssets;
use crate::config::Config;
use crate::simulation::SpritePackAssets;
use crate::state::editor_screen::EditorScreenAssets;
use crate::state::AppState::*;
use crate::ui::FontSize;
use crate::ui::InteractionPalette;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::ui::HEADER_FONT_HANDLE;
use crate::AppRoot;

pub struct TitleScreenStatePlugin;

impl Plugin for TitleScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TitleScreenConfig>()
            .register_type::<TitleScreenAssets>()
            .init_collection::<TitleScreenAssets>()
            .add_loading_state(LoadingState::new(TitleScreen))
            .add_collection_to_loading_state::<_, EditorScreenAssets>(TitleScreen)
            .add_collection_to_loading_state::<_, SpritePackAssets>(TitleScreen)
            .add_collection_to_loading_state::<_, AudioAssets>(TitleScreen)
            .add_plugins(ProgressPlugin::new(TitleScreen))
            .add_systems(OnEnter(TitleScreen), enter_title_screen)
            .add_systems(OnExit(TitleScreen), exit_title_screen);
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct TitleScreenConfig {
    border_color: Color,
    border_width: Val,
    background_color: Color,
    text_color: Color,
    hyperlink_text_color: Color,
    font_size: Val,

    title_background_color: Color,
    title_font_size: Val,

    button_width: Val,
    button_normal_color: Color,
    button_hovered_color: Color,
    button_pressed_color: Color,
    button_text_color: Color,
    button_font_size: Val,
}

const TITLE_TEXT: &str = "Bevy Jam Simulator";
// bevy_text cringe
const BODY_TEXT: [[&str; 4]; 4] = [
    ["Welcome to the fourth official Bevy Jam!", "", "", ""],
    [
        "In this 9 day event, your goal is to make a game in ",
        "Bevy Engine",
        ",",
        "",
    ],
    [
        "the free and open-source game engine built in Rust.",
        "",
        "",
        "",
    ],
    ["The theme is: ", "", "", "That's a LOT of entities!"],
];

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TitleScreenAssets {
    // TODO: Music / SFX maybe
}

fn enter_title_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    let config = &config.title_screen;
    commands.insert_resource(ClearColor(config.background_color));

    let screen = commands
        .spawn((
            Name::new("TitleScreen"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(VMin(2.5)),
                    ..default()
                },
                background_color: config.background_color.into(),
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    let container = commands
        .spawn((
            Name::new("Container"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    align_items: AlignItems::Center,
                    border: UiRect::all(config.border_width),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.background_color.into(),
                border_color: config.border_color.into(),
                ..default()
            },
        ))
        .set_parent(screen)
        .id();

    let title_container = commands
        .spawn((
            Name::new("TitleContainer"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::vertical(Vh(4.4)),
                    border: UiRect::bottom(config.border_width),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.title_background_color.into(),
                border_color: config.border_color.into(),
                ..default()
            },
        ))
        .set_parent(container)
        .id();

    commands
        .spawn((
            Name::new("TitleText"),
            TextBundle::from_section(
                TITLE_TEXT,
                TextStyle {
                    font: HEADER_FONT_HANDLE,
                    color: config.text_color,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Center),
            FontSize::new(config.title_font_size),
        ))
        .set_parent(title_container);

    let body_container = commands
        .spawn((
            Name::new("BodyContainer"),
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::axes(Vw(1.9), Vh(5.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Vh(4.0),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(container)
        .id();

    // bevy_text cringe
    for (i, line) in BODY_TEXT.into_iter().enumerate() {
        commands
            .spawn((
                Name::new(format!("BodyTextLine{i}")),
                TextBundle::from_sections([
                    TextSection::new(
                        line[0],
                        TextStyle {
                            font: FONT_HANDLE,
                            color: config.text_color,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        line[1],
                        TextStyle {
                            font: FONT_HANDLE,
                            color: config.hyperlink_text_color,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        line[2],
                        TextStyle {
                            font: FONT_HANDLE,
                            color: config.text_color,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        line[3],
                        TextStyle {
                            font: BOLD_FONT_HANDLE,
                            color: config.text_color,
                            ..default()
                        },
                    ),
                ]),
                FontSize::new(config.font_size),
            ))
            .set_parent(body_container);
    }

    let join_button = commands
        .spawn((
            Name::new("JoinButton"),
            ButtonBundle {
                style: Style {
                    width: config.button_width,
                    margin: UiRect::top(Vh(6.1)),
                    padding: UiRect::all(VMin(3.3)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: config.button_normal_color.into(),
                ..default()
            },
            InteractionPalette {
                normal: config.button_normal_color,
                hovered: config.button_hovered_color,
                pressed: config.button_pressed_color,
                disabled: Color::NONE,
            },
            On::<Pointer<Click>>::run(
                |mut next_state: ResMut<NextState<_>>, progress: Res<ProgressCounter>| {
                    let Progress { done, total } = progress.progress_complete();
                    next_state.set(if done >= total {
                        EditorScreen
                    } else {
                        LoadingScreen
                    });
                },
            ),
        ))
        .set_parent(container)
        .id();

    commands
        .spawn((
            Name::new("JoinButtonText"),
            TextBundle::from_section(
                "Join",
                TextStyle {
                    font: HEADER_FONT_HANDLE,
                    color: config.button_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.button_font_size),
        ))
        .set_parent(join_button);
}

fn exit_title_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.entity(root.ui).despawn_descendants();
}
