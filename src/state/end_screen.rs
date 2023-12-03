use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;

use crate::config::Config;
use crate::state::AppState;
use crate::state::AppState::*;
use crate::ui::vh;
use crate::ui::vmin;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::AppRoot;

pub struct EndScreenStatePlugin;

impl Plugin for EndScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EndScreenAssets>()
            .init_collection::<EndScreenAssets>()
            .init_resource::<ActionState<EndScreenAction>>()
            .add_plugins(InputManagerPlugin::<EndScreenAction>::default())
            .add_systems(OnEnter(EndScreen), enter_end_screen)
            .add_systems(OnExit(EndScreen), exit_end_screen)
            .add_systems(
                Update,
                (
                    end_screen_action_restart.run_if(action_just_pressed(EndScreenAction::Restart)),
                    end_screen_action_quit.run_if(action_just_pressed(EndScreenAction::Quit)),
                ),
            );
    }
}

const BACKGROUND_COLOR: Color = Color::rgb(0.067, 0.067, 0.067);
const BORDER_COLOR: Color = Color::rgb(0.161, 0.161, 0.161);
const BORDER_WIDTH: Val = Val::VMin(0.9);

const TITLE_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: Color::rgb(0.737, 0.737, 0.737),
};
const TITLE_TEXT: &str = "Results";
const TITLE_FONT_SIZE: Val = Val::Vw(5.0);

const TABLE_HEADER_BACKGROUND_COLOR: Color = Color::rgb(0.106, 0.106, 0.106);
const TABLE_HEADER_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: Color::rgb(0.624, 0.624, 0.624),
};
const TABLE_HEADER_TEXT: [&str; 4] = ["Criteria", "Rank", "Score", "Raw Score"];

const _TABLE_TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: 0.0,
    color: Color::rgb(0.737, 0.737, 0.737),
};
const TABLE_FONT_SIZE: Val = Val::Vw(3.0);
const _TABLE_CRITERIA_TEXT: [&str; 6] = [
    "Fun",
    "Presentation",
    "Theme Interpretation",
    "Entities",
    "Lines of Code",
    "Overall",
];

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EndScreenAssets {}

#[derive(Actionlike, Reflect, PartialEq, Eq, Hash, Clone)]
enum EndScreenAction {
    Restart,
    Quit,
}

fn enter_end_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    commands.insert_resource(ClearColor(config.bg_color));

    commands.insert_resource(
        InputMap::default()
            .insert(MouseButton::Left, EndScreenAction::Restart)
            .insert(GamepadButtonType::Start, EndScreenAction::Restart)
            .insert(KeyCode::Return, EndScreenAction::Restart)
            .insert(KeyCode::Space, EndScreenAction::Restart)
            .insert(KeyCode::Escape, EndScreenAction::Quit)
            .insert(KeyCode::Q, EndScreenAction::Quit)
            .build(),
    );

    let screen = commands
        .spawn((
            Name::new("EndScreen"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::new(vmin(15.0), vmin(15.0), vh(7.0), vmin(15.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BACKGROUND_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    commands
        .spawn((
            Name::new("TitleText"),
            TextBundle::from_section(TITLE_TEXT, TITLE_TEXT_STYLE),
            FontSize::new(TITLE_FONT_SIZE),
        ))
        .set_parent(screen);

    let table = commands
        .spawn((
            Name::new("Table"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    margin: UiRect::top(vh(5.0)),
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

    let header_row = commands
        .spawn((
            Name::new("TableHeader"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    border: UiRect::bottom(BORDER_WIDTH),
                    ..default()
                },
                background_color: TABLE_HEADER_BACKGROUND_COLOR.into(),
                border_color: BORDER_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(table)
        .id();

    for (i, &header) in TABLE_HEADER_TEXT.iter().enumerate() {
        commands
            .spawn((
                Name::new(format!("TableHeaderCol{}", i)),
                TextBundle::from_section(header, TABLE_HEADER_TEXT_STYLE),
                FontSize::new(TABLE_FONT_SIZE),
            ))
            .set_parent(header_row);
    }
}

fn exit_end_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.remove_resource::<InputMap<EndScreenAction>>();
    commands.entity(root.ui).despawn_descendants();
}

fn end_screen_action_restart(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(TitleScreen);
}

fn end_screen_action_quit(mut app_exit: EventWriter<AppExit>) {
    app_exit.send(AppExit);
}
