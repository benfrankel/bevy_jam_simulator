use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::config::Config;
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
            .add_systems(OnEnter(EndScreen), enter_end_screen)
            .add_systems(OnExit(EndScreen), exit_end_screen);
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

const TABLE_TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: 0.0,
    color: Color::rgb(0.737, 0.737, 0.737),
};
const TABLE_FONT_SIZE: Val = Val::Vw(2.5);
const TABLE_CRITERIA_TEXT: [&str; 6] = [
    "Fun",
    "Presentation",
    "Theme Interpretation",
    "Entities",
    "Lines of Code",
    "Overall",
];

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EndScreenAssets {
    // TODO: Music / SFX maybe
}

fn enter_end_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    commands.insert_resource(ClearColor(config.bg_color));

    let screen = commands
        .spawn((
            Name::new("EndScreen"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::new(vmin(15.0), vmin(15.0), vh(7.0), Val::ZERO),
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
                    display: Display::Grid,
                    width: Val::Percent(100.0),
                    margin: UiRect::top(vh(10.0)),
                    border: UiRect::all(BORDER_WIDTH),
                    // FIXME: For some reason all the extra space goes to the first column
                    grid_template_columns: vec![GridTrack::auto(); 4],
                    ..default()
                },
                background_color: BACKGROUND_COLOR.into(),
                border_color: BORDER_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(screen)
        .id();

    for (i, &entry) in TABLE_HEADER_TEXT.iter().enumerate() {
        let cell = commands
            .spawn((
                Name::new(format!("HeaderCell{i}")),
                NodeBundle {
                    style: Style {
                        padding: UiRect::all(vmin(3.5)),
                        ..default()
                    },
                    background_color: TABLE_HEADER_BACKGROUND_COLOR.into(),
                    ..default()
                },
            ))
            .set_parent(table)
            .id();
        commands
            .spawn((
                Name::new("CellText"),
                TextBundle::from_section(entry, TABLE_HEADER_TEXT_STYLE),
                FontSize::new(TABLE_FONT_SIZE),
            ))
            .set_parent(cell);
    }

    for (row, &criteria) in TABLE_CRITERIA_TEXT.iter().enumerate() {
        // TODO: Populate cells based on resource values like entity count / lines of code
        let entries = vec![criteria, "#13", "4.233", "4.233"];
        for (col, &text) in entries.iter().enumerate() {
            let cell = commands
                .spawn((
                    Name::new(format!("BodyCellRow{row}Col{col}")),
                    NodeBundle {
                        style: Style {
                            padding: UiRect::all(vmin(3.5)),
                            border: UiRect::top(BORDER_WIDTH),
                            ..default()
                        },
                        border_color: BORDER_COLOR.into(),
                        ..default()
                    },
                ))
                .set_parent(table)
                .id();
            commands
                .spawn((
                    Name::new("CellText"),
                    TextBundle::from_section(text, TABLE_TEXT_STYLE),
                    FontSize::new(TABLE_FONT_SIZE),
                ))
                .set_parent(cell);
        }
    }
}

fn exit_end_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.entity(root.ui).despawn_descendants();
}
