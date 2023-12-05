use bevy::prelude::*;

use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::CodeTyper;
use crate::ui::FontSize;
use crate::ui::FONT_HANDLE;

/// Spawns the fake code panel with light theme.
pub fn spawn_light_code_panel(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let code_panel = commands
        .spawn((
            Name::new("CodePanel"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: config.code_panel_height,
                    padding: UiRect::all(Val::VMin(2.0)),
                    ..default()
                },
                background_color: config.light_theme_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("CodePanelText"),
            TextBundle::from_section(
                "Aargh, my eyes!\n\nI cannot work with this light theme!\n\nI need to install a dark theme from the panel on the right.\n\nThe installed upgrades will appear on the left panel.",
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.light_theme_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.code_panel_font_size),
        ))
        .set_parent(code_panel);

    code_panel
}

pub fn spawn_code_panel(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let code_panel = commands
        .spawn((
            Name::new("CodePanel"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: config.code_panel_height,
                    padding: UiRect::all(Val::VMin(2.0)),
                    ..default()
                },
                background_color: config.code_panel_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("CodePanelText"),
            TextBundle::from_section(
                "// Start typing to generate lines of code!\n",
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.code_panel_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.code_panel_font_size),
            CodeTyper {
                lines_count: 2,
                lines_max: config.code_panel_lines_max,
                ..default()
            },
        ))
        .set_parent(code_panel);

    code_panel
}
