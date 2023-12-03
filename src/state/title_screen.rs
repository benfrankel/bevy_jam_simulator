use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::prelude::*;
use iyes_progress::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::state::editor_screen::EditorScreenAssets;
use crate::state::AppState;
use crate::state::AppState::*;
use crate::ui::vh;
use crate::ui::vmin;
use crate::ui::vw;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::AppRoot;

pub struct TitleScreenStatePlugin;

impl Plugin for TitleScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TitleScreenAssets>()
            .init_collection::<TitleScreenAssets>()
            .add_loading_state(LoadingState::new(TitleScreen))
            .add_collection_to_loading_state::<_, EditorScreenAssets>(TitleScreen)
            .add_plugins(ProgressPlugin::new(TitleScreen))
            .add_systems(OnEnter(TitleScreen), enter_title_screen)
            .add_systems(OnExit(TitleScreen), exit_title_screen)
            .add_systems(Update, update_button.run_if(in_state(TitleScreen)));
    }
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct TitleScreenConfig {
    text_color: Color,
    border_color: Color,
    border_width: Val,
    background_color: Color,

    title_background_color: Color,
    title_font_size: Val,

    body_font_size: Val,

    button_text_color: Color,
    button_font_size: Val,
    button_border_color: Color,
    button_normal_color: Color,
    button_hovered_color: Color,
    button_pressed_color: Color,
}

impl Default for TitleScreenConfig {
    fn default() -> Self {
        Self {
            text_color: Color::BLACK,
            border_color: Color::BLACK,
            border_width: Val::Px(1.0),
            background_color: Color::WHITE,
            title_background_color: Color::GRAY,
            title_font_size: Val::Px(10.0),
            body_font_size: Val::Px(10.0),
            button_text_color: Color::WHITE,
            button_font_size: Val::Px(10.0),
            button_border_color: Color::BLACK,
            button_normal_color: Color::BLUE,
            button_hovered_color: Color::CYAN,
            button_pressed_color: Color::MIDNIGHT_BLUE,
        }
    }
}

const TITLE_TEXT: &str = "Bevy Jam #4: The Game";
const BODY_TEXT: &str = "Welcome to the fourth official Bevy Jam!\n \nIn this 9 day event, your goal is to make a game in Bevy Engine,\nthe free and open-source game engine built in Rust.\n \nThe theme is: That's a LOT of Entities!";
const THEME: &str = "That's a LOT of Entities!";

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TitleScreenAssets {
    // TODO: Music / SFX maybe
}

fn enter_title_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    let config = &config.title_screen;
    let text_style = TextStyle {
        font: FONT_HANDLE,
        color: config.text_color,
        ..default()
    };
    let bold_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.text_color,
        ..default()
    };
    let button_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.button_text_color,
        ..default()
    };

    commands.insert_resource(ClearColor(config.background_color));

    let screen = commands
        .spawn((
            Name::new("TitleScreen"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(vmin(4.5)),
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
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
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
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::vertical(vh(8.0)),
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
            TextBundle::from_section(TITLE_TEXT, bold_text_style.clone())
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
                    margin: UiRect::axes(vw(6.0), vh(9.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: vh(2.5),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(container)
        .id();

    // Ugly workaround to be able to customize line spacing
    for (i, line) in BODY_TEXT.lines().enumerate() {
        // Ugly workaround to put the theme in bold
        let mut sections = vec![];
        for (j, section) in line.split(THEME).enumerate() {
            sections.push(TextSection::new(section, text_style.clone()));
            if j > 0 {
                sections.push(TextSection::new(THEME, bold_text_style.clone()));
            }
        }

        commands
            .spawn((
                Name::new(format!("BodyTextLine{i}")),
                TextBundle::from_sections(sections),
                FontSize::new(config.body_font_size),
            ))
            .set_parent(body_container);
    }

    let join_button = commands
        .spawn((
            Name::new("JoinButton"),
            ButtonBundle {
                style: Style {
                    margin: UiRect::top(vh(12.0)),
                    padding: UiRect::axes(vw(10.0), vh(6.0)),
                    border: UiRect::all(config.border_width),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: config.button_normal_color.into(),
                border_color: config.button_border_color.into(),
                ..default()
            },
            On::<Pointer<Click>>::run(
                |mut next_state: ResMut<NextState<AppState>>, progress: Res<ProgressCounter>| {
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
            TextBundle::from_section("Join", button_text_style),
            FontSize::new(config.button_font_size),
        ))
        .set_parent(join_button);
}

fn exit_title_screen(mut commands: Commands, root: Res<AppRoot>) {
    // TODO: This and the other despawn_decendants() should probably make use of DespawnSet...
    commands.entity(root.ui).despawn_descendants();
}

fn update_button(
    config: Res<Config>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let config = &config.title_screen;
    for (interaction, mut color) in &mut interaction_query {
        *color = match interaction {
            Interaction::Pressed => config.button_pressed_color,
            Interaction::Hovered => config.button_hovered_color,
            Interaction::None => config.button_normal_color,
        }
        .into()
    }
}
