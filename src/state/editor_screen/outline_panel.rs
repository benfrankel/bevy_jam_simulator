use bevy::math::vec2;
use bevy::prelude::*;

use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::FontSize;
use crate::ui::InteractionColor;
use crate::ui::Tooltip;
use crate::ui::TooltipSide;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;

// TODO: Add scrollbar
pub fn spawn_outline_panel(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let outline_panel = commands
        .spawn((
            Name::new("OutlinePanel"),
            NodeBundle {
                style: Style {
                    min_width: config.outline_panel_width,
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.outline_panel_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("OutlineHeader"),
            TextBundle {
                text: Text::from_section(
                    "Outline",
                    TextStyle {
                        font: BOLD_FONT_HANDLE,
                        color: config.outline_panel_text_color,
                        ..default()
                    },
                ),
                style: Style {
                    // Hiding this because it looks bad :(
                    display: Display::None,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
            FontSize::new(config.outline_panel_header_font_size),
        ))
        .set_parent(outline_panel);

    // TODO: Replace these dummy plugins
    for plugin_name in ["FooPlugin", "BarPlugin", "QuuxPlugin"] {
        let plugin = commands
            .spawn((
                Name::new("Plugin"),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        margin: UiRect::bottom(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                },
                Interaction::default(),
                InteractionColor {
                    normal: Color::NONE,
                    hovered: config.outline_panel_highlight_color,
                    pressed: config.outline_panel_highlight_color,
                },
                Tooltip {
                    text: format!("This is the description for {plugin_name}."),
                    side: TooltipSide::Right,
                    offset: vec2(12.0, 0.0),
                },
            ))
            .set_parent(outline_panel)
            .id();

        commands
            .spawn((
                Name::new("PluginText"),
                TextBundle::from_section(
                    plugin_name,
                    TextStyle {
                        font: FONT_HANDLE,
                        color: config.outline_panel_text_color,
                        ..default()
                    },
                ),
                FontSize::new(config.outline_panel_font_size),
            ))
            .set_parent(plugin);
    }

    outline_panel
}
