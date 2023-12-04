use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::Deserialize;
use serde::Serialize;

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
        app.register_type::<EndScreenConfig>()
            .register_type::<EndScreenAssets>()
            .init_collection::<EndScreenAssets>()
            .add_systems(OnEnter(EndScreen), enter_end_screen)
            .add_systems(OnExit(EndScreen), exit_end_screen);
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct EndScreenConfig {
    background_color: Color,
    border_color: Color,
    border_width: Val,

    title_text_color: Color,
    title_font_size: Val,

    table_header_background_color: Color,
    table_header_text_color: Color,
    table_text_color: Color,
    table_font_size: Val,
}

const TITLE_TEXT: &str = "Results";
const TABLE_HEADER_TEXT: [&str; 4] = ["Criteria", "Rank", "Score", "Raw Score"];
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
    let config = &config.end_screen;
    let title_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.title_text_color,
        ..default()
    };
    let table_header_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.table_header_text_color,
        ..default()
    };
    let table_text_style = TextStyle {
        font: FONT_HANDLE,
        color: config.table_text_color,
        ..default()
    };

    commands.insert_resource(ClearColor(config.background_color));

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
                background_color: config.background_color.into(),
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    commands
        .spawn((
            Name::new("TitleText"),
            TextBundle::from_section(TITLE_TEXT, title_text_style),
            FontSize::new(config.title_font_size),
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
                    border: UiRect::all(config.border_width),
                    // FIXME: For some reason all the extra space goes to the first column
                    grid_template_columns: vec![GridTrack::auto(); 4],
                    ..default()
                },
                background_color: config.background_color.into(),
                border_color: config.border_color.into(),
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
                    background_color: config.table_header_background_color.into(),
                    ..default()
                },
            ))
            .set_parent(table)
            .id();
        commands
            .spawn((
                Name::new("CellText"),
                TextBundle::from_section(entry, table_header_text_style.clone()),
                FontSize::new(config.table_font_size),
            ))
            .set_parent(cell);
    }

    for (row, &criterion) in TABLE_CRITERIA_TEXT.iter().enumerate() {
        // TODO: Populate cells based on resource values like entity count / lines of code
        let entries = [criterion, "#13", "4.233", "4.233"];
        for (col, &text) in entries.iter().enumerate() {
            let cell = commands
                .spawn((
                    Name::new(format!("BodyCellRow{row}Col{col}")),
                    NodeBundle {
                        style: Style {
                            padding: UiRect::all(vmin(3.5)),
                            border: UiRect::top(config.border_width),
                            ..default()
                        },
                        border_color: config.border_color.into(),
                        ..default()
                    },
                ))
                .set_parent(table)
                .id();
            commands
                .spawn((
                    Name::new("CellText"),
                    TextBundle::from_section(text, table_text_style.clone()),
                    FontSize::new(config.table_font_size),
                ))
                .set_parent(cell);
        }
    }
}

fn exit_end_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.entity(root.ui).despawn_descendants();
}
