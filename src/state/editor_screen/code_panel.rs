use bevy::prelude::*;
use bevy::ui::Val::*;

use super::EditorScreenConfig;
use crate::state::editor_screen::EditorScreenTheme;
use crate::ui::CodeTyper;
use crate::ui::FontSize;
use crate::ui::FONT_HANDLE;

const LIGHT_THEME_TEXT: &str = "Aargh, my eyes!

I cannot work with this light theme!

I need to install a dark theme from the panel on the right.

The installed upgrades will appear on the left panel.";

/// Spawns the fake code panel with light theme.
pub fn spawn_light_code_panel(
    commands: &mut Commands,
    config: &EditorScreenConfig,
    theme: &EditorScreenTheme,
) -> Entity {
    let code_panel = commands
        .spawn((
            Name::new("CodePanel"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    min_height: config.code_panel_height,
                    padding: UiRect::all(VMin(2.0)),
                    ..default()
                },
                background_color: theme.code_panel_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("CodePanelText"),
            TextBundle::from_section(
                LIGHT_THEME_TEXT,
                TextStyle {
                    font: FONT_HANDLE,
                    color: theme.code_panel_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.code_panel_font_size),
        ))
        .set_parent(code_panel);

    code_panel
}

pub fn spawn_code_panel(
    commands: &mut Commands,
    config: &EditorScreenConfig,
    theme: &EditorScreenTheme,
) -> Entity {
    let code_panel = commands
        .spawn((
            Name::new("CodePanel"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    min_height: config.code_panel_height,
                    padding: UiRect::all(VMin(2.0)),
                    ..default()
                },
                background_color: theme.code_panel_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("CodePanelText"),
            TextBundle::from_section(
                "// Start typing...",
                TextStyle {
                    font: FONT_HANDLE,
                    color: theme.code_panel_text_color,
                    ..default()
                },
            )
            .with_no_wrap(),
            FontSize::new(config.code_panel_font_size),
            CodeTyper {
                lines_count: 1,
                lines_max: config.code_panel_lines_max,
                ..default()
            },
        ))
        .set_parent(code_panel);

    code_panel
}
