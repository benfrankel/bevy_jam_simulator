use bevy::prelude::*;

use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::CodeTyper;
use crate::ui::FontSize;
use crate::ui::FONT_HANDLE;

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
                "// Start typing to generate lines of code!\n\n",
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.code_panel_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.code_panel_font_size),
            CodeTyper {
                lines_count: 3,
                lines_max: config.code_panel_lines_max,
                ..default()
            },
        ))
        .set_parent(code_panel);

    code_panel
}