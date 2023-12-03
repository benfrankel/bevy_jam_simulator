use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use iyes_progress::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;

use crate::config::Config;
use crate::state::game::GameAssets;
use crate::state::AppState;
use crate::state::AppState::*;
use crate::ui::vh;
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
            .init_resource::<ActionState<TitleScreenAction>>()
            .add_plugins(InputManagerPlugin::<TitleScreenAction>::default())
            .add_systems(OnEnter(TitleScreen), enter_title_screen)
            .add_systems(OnExit(TitleScreen), exit_title_screen)
            .add_systems(
                Update,
                (
                    title_screen_action_start
                        .run_if(action_just_pressed(TitleScreenAction::Start))
                        .after(TrackedProgressSet),
                    title_screen_action_quit.run_if(action_just_pressed(TitleScreenAction::Quit)),
                ),
            );
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
const BORDER_WIDTH: f32 = 1.5;

const HEADER_BACKGROUND_COLOR: Color = Color::rgb(0.549, 0.647, 0.796);
const HEADER_FONT_SIZE: f32 = 12.0;
const HEADER_TEXT: &str = "Bevy Jam #4: The Game";

const BODY_FONT_SIZE: f32 = 7.0;
const BODY_TEXT: &str = "Welcome to the fourth official Bevy Jam!\n \nIn this 9 day event, your goal is to make a game in Bevy Engine,\nthe free and open-source game engine built in Rust.\n \nThe theme is: That's a LOT of Entities!";
const THEME: &str = "That's a LOT of Entities!";

const BUTTON_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: Color::WHITE,
};
const BUTTON_FONT_SIZE: f32 = 12.0;
const BUTTON_BACKGROUND_COLOR: Color = Color::rgb(0.000, 0.188, 0.702);
const BUTTON_BORDER_COLOR: Color = Color::rgb(0.118, 0.306, 0.820);

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct TitleScreenAssets {}

#[derive(Actionlike, Reflect, PartialEq, Eq, Hash, Clone)]
enum TitleScreenAction {
    Start,
    Quit,
}

fn enter_title_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    commands.insert_resource(ClearColor(config.bg_color));

    commands.insert_resource(
        InputMap::default()
            .insert(MouseButton::Left, TitleScreenAction::Start)
            .insert(GamepadButtonType::Start, TitleScreenAction::Start)
            .insert(KeyCode::Return, TitleScreenAction::Start)
            .insert(KeyCode::Space, TitleScreenAction::Start)
            .insert(KeyCode::Escape, TitleScreenAction::Quit)
            .insert(KeyCode::Q, TitleScreenAction::Quit)
            .build(),
    );

    let screen = commands
        .spawn((
            Name::new("TitleScreen"),
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

    let container = commands
        .spawn((
            Name::new("Container"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    border: UiRect::axes(vw(BORDER_WIDTH), vh(BORDER_WIDTH)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor(BACKGROUND_COLOR),
                border_color: BorderColor(BORDER_COLOR),
                ..default()
            },
        ))
        .set_parent(screen)
        .id();

    let header_container = commands
        .spawn((
            Name::new("HeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: vh(40.0),
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::bottom(vh(BORDER_WIDTH)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BackgroundColor(HEADER_BACKGROUND_COLOR),
                border_color: BorderColor(BORDER_COLOR),
                ..default()
            },
        ))
        .set_parent(container)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section(HEADER_TEXT, BOLD_TEXT_STYLE)
                .with_text_alignment(TextAlignment::Center),
            FontSize::new(vh(HEADER_FONT_SIZE)),
        ))
        .set_parent(header_container);

    let body_container = commands
        .spawn((
            Name::new("BodyContainer"),
            NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_items: JustifyItems::Center,
                    align_content: AlignContent::Center,
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
                Name::new(format!("BodyTextLine{}", i)),
                TextBundle::from_sections(sections),
                FontSize::new(vh(BODY_FONT_SIZE)),
            ))
            .set_parent(body_container);
    }

    let join_button = commands
        .spawn((
            Name::new("JoinButton"),
            ButtonBundle {
                style: Style {
                    margin: UiRect::top(vh(10.0)),
                    padding: UiRect::axes(vw(6.0), vh(4.0)),
                    border: UiRect::axes(vw(BORDER_WIDTH), vh(BORDER_WIDTH)),
                    ..default()
                },
                background_color: BackgroundColor(BUTTON_BACKGROUND_COLOR),
                border_color: BorderColor(BUTTON_BORDER_COLOR),
                ..default()
            },
        ))
        .set_parent(container)
        .id();

    commands
        .spawn((
            Name::new("JoinButtonText"),
            TextBundle::from_section("Join", BUTTON_TEXT_STYLE),
            FontSize::new(vh(BUTTON_FONT_SIZE)),
        ))
        .set_parent(join_button);
}

fn exit_title_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.remove_resource::<InputMap<TitleScreenAction>>();
    // TODO: This and the other despawn_decendants() should probably make use of DespawnSet...
    commands.entity(root.ui).despawn_descendants();
}

fn title_screen_action_start(
    mut next_state: ResMut<NextState<AppState>>,
    progress: Res<ProgressCounter>,
) {
    // Show loading screen only if assets are still loading
    let Progress { done, total } = progress.progress_complete();
    //next_state.set(if done >= total { Game } else { LoadingScreen });
}

fn title_screen_action_quit(mut app_exit: EventWriter<AppExit>) {
    app_exit.send(AppExit);
}
