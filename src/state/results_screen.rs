use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::physics::PhysicsSettings;
use crate::simulation::PassiveCodeTyper;
use crate::simulation::PassiveEntitySpawner;
use crate::simulation::Simulation;
use crate::state::editor_screen::EditorScreenStartTime;
use crate::state::editor_screen::UpgradeOutline;
use crate::state::AppState;
use crate::state::AppState::*;
use crate::ui::FontSize;
use crate::ui::InteractionPalette;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::ui::HEADER_FONT_HANDLE;
use crate::AppRoot;

pub struct ResultsScreenStatePlugin;

impl Plugin for ResultsScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ResultsScreenConfig>()
            .register_type::<ResultsScreenAssets>()
            .init_collection::<ResultsScreenAssets>()
            .add_systems(OnEnter(ResultsScreen), enter_results_screen)
            .add_systems(OnExit(ResultsScreen), exit_results_screen);
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct ResultsScreenConfig {
    background_color: Color,
    border_color: Color,
    border_width: Val,
    text_color: Color,
    hyperlink_text_color: Color,
    font_size: Val,

    title_text_color: Color,
    title_font_size: Val,

    table_header_background_color: Color,
    table_header_text_color: Color,

    return_button_normal_color: Color,
    return_button_hovered_color: Color,
    return_button_pressed_color: Color,
    return_button_text_color: Color,
}

const TITLE_TEXT: &str = "Results";
const TABLE_HEADER_TEXT: [&str; 4] = ["Criteria", "Rank", "Score*", "Raw Score"];
const TABLE_CRITERIA_TEXT: [&str; 4] = ["Fun", "Presentation", "Theme Interpretation", "Overall"];

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ResultsScreenAssets {
    // TODO: Music / SFX maybe
}

fn enter_results_screen(
    mut commands: Commands,
    root: Res<AppRoot>,
    config: Res<Config>,
    simulation: Res<Simulation>,
    start_time: Res<EditorScreenStartTime>,
    time: Res<Time>,
) {
    let config = &config.results_screen;
    commands.insert_resource(ClearColor(config.background_color));

    let screen = commands
        .spawn((
            Name::new("ResultsScreen"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::new(VMin(8.3), VMin(8.3), Vh(4.0), Val::ZERO),
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
            TextBundle::from_section(
                TITLE_TEXT,
                TextStyle {
                    font: HEADER_FONT_HANDLE,
                    color: config.title_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.title_font_size),
        ))
        .set_parent(screen);

    let table = commands
        .spawn((
            Name::new("Table"),
            NodeBundle {
                style: Style {
                    display: Display::Grid,
                    width: Percent(100.0),
                    margin: UiRect::top(Vh(5.5)),
                    border: UiRect::all(config.border_width),
                    column_gap: Px(-1.0),
                    grid_template_columns: vec![
                        GridTrack::auto(),
                        GridTrack::fr(1.0),
                        GridTrack::fr(1.0),
                        GridTrack::fr(1.5),
                    ],
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
                        padding: UiRect::all(VMin(2.0)),
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
                TextBundle::from_section(
                    entry,
                    TextStyle {
                        font: BOLD_FONT_HANDLE,
                        color: config.table_header_text_color,
                        ..default()
                    },
                ),
                FontSize::new(config.font_size),
            ))
            .set_parent(cell);
    }

    const SUBMISSIONS: f64 = 90.0;
    const LO: f64 = 1.5;
    const HI: f64 = 4.8;
    let elapsed = time.elapsed_seconds_f64() - start_time.0;
    let ratings = (elapsed / 60.0).clamp(5.0, 120.0).floor();
    let scores: [f64; 4] = simulation.calculate_scores(ratings);
    for (row, (&criterion, score)) in TABLE_CRITERIA_TEXT
        .iter()
        .zip(scores.into_iter())
        .enumerate()
    {
        // Calculate rank by linearly mapping score from [LO, HI] to [1, SUBMISSIONS]
        let rank = (1.0 - (score.clamp(LO, HI) - LO) / (HI - LO)) * (SUBMISSIONS - 1.0) + 1.0;

        let entries = [
            criterion,
            &format!("#{:.0}", rank),
            &format!("{:.3}", score),
            &format!("{:.3}", score),
        ];
        for (col, &text) in entries.iter().enumerate() {
            let cell = commands
                .spawn((
                    Name::new(format!("BodyCellRow{row}Col{col}")),
                    NodeBundle {
                        style: Style {
                            padding: UiRect::all(VMin(3.0)),
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
                    TextBundle::from_section(
                        text,
                        TextStyle {
                            font: FONT_HANDLE,
                            color: if col == 0 && row != 3 {
                                config.hyperlink_text_color
                            } else {
                                config.text_color
                            },
                            ..default()
                        },
                    ),
                    FontSize::new(config.font_size),
                ))
                .set_parent(cell);
        }
    }

    let hbox = commands
        .spawn((
            Name::new("HBox"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    margin: UiRect::top(Vh(4.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(screen)
        .id();

    commands
        .spawn((
            Name::new("RankedText"),
            TextBundle::from_section(
                format!("Ranked from {ratings:.0} ratings."),
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.text_color,
                    ..default()
                },
            ),
            FontSize::new(config.font_size),
        ))
        .set_parent(hbox);

    let return_button = spawn_return_button(&mut commands, config);
    commands.entity(return_button).set_parent(hbox);
}

fn exit_results_screen(mut commands: Commands, root: Res<AppRoot>) {
    commands.entity(root.ui).despawn_descendants();

    // Reset resources so replaying works
    commands.insert_resource(Simulation::default());
    commands.insert_resource(PhysicsSettings::default());
    commands.insert_resource(PassiveCodeTyper::default());
    commands.insert_resource(PassiveEntitySpawner::default());
    commands.insert_resource(UpgradeOutline::default());
}

fn spawn_return_button(commands: &mut Commands, config: &ResultsScreenConfig) -> Entity {
    let return_button = commands
        .spawn((
            Name::new("ReturnButton"),
            ButtonBundle {
                style: Style {
                    padding: UiRect::all(Px(16.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: config.return_button_normal_color.into(),
                ..default()
            },
            InteractionPalette {
                normal: config.return_button_normal_color,
                hovered: config.return_button_hovered_color,
                pressed: config.return_button_pressed_color,
                disabled: Color::NONE,
            },
            On::<Pointer<Click>>::run(|mut next_state: ResMut<NextState<_>>| {
                next_state.set(AppState::TitleScreen);
            }),
        ))
        .id();

    commands
        .spawn((
            Name::new("ReturnButtonText"),
            TextBundle::from_section(
                "Try another jam?",
                TextStyle {
                    font: HEADER_FONT_HANDLE,
                    color: config.return_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.font_size),
        ))
        .set_parent(return_button);

    return_button
}
