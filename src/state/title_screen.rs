use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::prelude::*;
use iyes_progress::prelude::*;

use crate::config::Config;
use crate::state::game::GameAssets;
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
            .add_collection_to_loading_state::<_, GameAssets>(TitleScreen)
            .add_plugins(ProgressPlugin::new(TitleScreen))
            .add_systems(OnEnter(TitleScreen), enter_title_screen)
            .add_systems(OnExit(TitleScreen), exit_title_screen)
            .add_systems(Update, update_button.run_if(in_state(TitleScreen)));
    }
}

const TEXT_COLOR: Color = Color::rgb(0.149, 0.149, 0.149);
const TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: 0.0,
    color: TEXT_COLOR,
};
const BOLD_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: TEXT_COLOR,
};

const BACKGROUND_COLOR: Color = Color::rgb(0.580, 0.682, 0.839);
const BORDER_COLOR: Color = Color::rgb(0.510, 0.612, 0.769);
const BORDER_WIDTH: Val = Val::VMin(1.0);

const TITLE_BACKGROUND_COLOR: Color = Color::rgb(0.549, 0.647, 0.796);
const TITLE_FONT_SIZE: Val = Val::Vw(4.5);
const TITLE_TEXT: &str = "Bevy Jam #4: The Game";

const BODY_FONT_SIZE: Val = Val::Vw(2.2);
const BODY_TEXT: &str = "Welcome to the fourth official Bevy Jam!\n \nIn this 9 day event, your goal is to make a game in Bevy Engine,\nthe free and open-source game engine built in Rust.\n \nThe theme is: That's a LOT of Entities!";
const THEME: &str = "That's a LOT of Entities!";

const BUTTON_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: Color::WHITE,
};
const BUTTON_FONT_SIZE: Val = Val::Vw(4.5);
const BUTTON_NORMAL_COLOR: Color = Color::rgb(0.000, 0.188, 0.702);
const BUTTON_HOVERED_COLOR: Color = Color::rgb(0.039, 0.227, 0.741);
const BUTTON_PRESSED_COLOR: Color = Color::rgb(0.000, 0.176, 0.690);
const BUTTON_BORDER_COLOR: Color = Color::rgb(0.118, 0.306, 0.820);

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TitleScreenAssets {
    // TODO: Music / SFX maybe
}

fn enter_title_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    commands.insert_resource(ClearColor(config.bg_color));

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
                background_color: BACKGROUND_COLOR.into(),
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
                    border: UiRect::all(BORDER_WIDTH),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BACKGROUND_COLOR.into(),
                border_color: BORDER_COLOR.into(),
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
                    border: UiRect::bottom(BORDER_WIDTH),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: TITLE_BACKGROUND_COLOR.into(),
                border_color: BORDER_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(container)
        .id();

    commands
        .spawn((
            Name::new("TitleText"),
            TextBundle::from_section(TITLE_TEXT, BOLD_TEXT_STYLE)
                .with_text_alignment(TextAlignment::Center),
            FontSize::new(TITLE_FONT_SIZE),
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
            sections.push(TextSection::new(section, TEXT_STYLE));
            if j > 0 {
                sections.push(TextSection::new(THEME, BOLD_TEXT_STYLE));
            }
        }

        commands
            .spawn((
                Name::new(format!("BodyTextLine{i}")),
                TextBundle::from_sections(sections),
                FontSize::new(BODY_FONT_SIZE),
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
                    border: UiRect::all(BORDER_WIDTH),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BUTTON_NORMAL_COLOR.into(),
                border_color: BUTTON_BORDER_COLOR.into(),
                ..default()
            },
            On::<Pointer<Click>>::run(
                |mut next_state: ResMut<NextState<AppState>>, progress: Res<ProgressCounter>| {
                    let Progress { done, total } = progress.progress_complete();
                    next_state.set(if done >= total { Game } else { LoadingScreen });
                },
            ),
        ))
        .set_parent(container)
        .id();

    commands
        .spawn((
            Name::new("JoinButtonText"),
            TextBundle::from_section("Join", BUTTON_TEXT_STYLE),
            FontSize::new(BUTTON_FONT_SIZE),
        ))
        .set_parent(join_button);
}

fn exit_title_screen(mut commands: Commands, root: Res<AppRoot>) {
    // TODO: This and the other despawn_decendants() should probably make use of DespawnSet...
    commands.entity(root.ui).despawn_descendants();
}

fn update_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match interaction {
            Interaction::Pressed => BUTTON_PRESSED_COLOR,
            Interaction::Hovered => BUTTON_HOVERED_COLOR,
            Interaction::None => BUTTON_NORMAL_COLOR,
        }
        .into()
    }
}
