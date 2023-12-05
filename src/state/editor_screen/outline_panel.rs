use bevy::math::vec2;
use bevy::prelude::*;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::state::editor_screen::EditorScreenTheme;
use crate::ui::FontSize;
use crate::ui::InteractionPalette;
use crate::ui::Tooltip;
use crate::ui::TooltipSide;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::upgrade::Upgrade;
use crate::upgrade::UpgradeEvent;
use crate::upgrade::UpgradeList;

pub struct OutlinePanelPlugin;

impl Plugin for OutlinePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsOutlineContainer>()
            .add_systems(Update, (add_upgrades_to_outline, update_outline_header));
    }
}

#[derive(Component, Reflect)]
struct IsOutlineContainer;

#[derive(Component, Reflect)]
struct IsOutlineHeader;

// TODO: Add scrollbar
pub fn spawn_outline_panel(commands: &mut Commands, theme: &EditorScreenTheme) -> Entity {
    let outline_panel = commands
        .spawn((
            Name::new("OutlinePanel"),
            NodeBundle {
                style: Style {
                    min_width: theme.outline_panel_width,
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: theme.outline_panel_background_color.into(),
                ..default()
            },
            IsOutlineContainer,
        ))
        .id();

    commands
        .spawn((
            Name::new("OutlineHeader"),
            TextBundle {
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: BOLD_FONT_HANDLE,
                        color: theme.outline_panel_text_color,
                        ..default()
                    },
                ),
                style: Style {
                    // Hiding this because it looks bad :(
                    // display: Display::None,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                ..default()
            },
            FontSize::new(theme.outline_panel_header_font_size),
            IsOutlineHeader,
        ))
        .set_parent(outline_panel);

    outline_panel
}

fn spawn_outline_entry(
    commands: &mut Commands,
    theme: &EditorScreenTheme,
    upgrade: &Upgrade,
) -> Entity {
    let outline_entry = commands
        .spawn((
            Name::new("OutlineEntry"),
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
            InteractionPalette {
                normal: Color::NONE,
                hovered: theme.outline_panel_highlight_color,
                pressed: theme.outline_panel_highlight_color,
                disabled: Color::NONE,
            },
            Tooltip {
                text: upgrade.description.clone(),
                side: TooltipSide::Right,
                offset: vec2(12.0, 0.0),
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("OutlineEntryText"),
            TextBundle::from_section(
                upgrade.name.clone(),
                TextStyle {
                    font: FONT_HANDLE,
                    color: theme.outline_panel_text_color,
                    ..default()
                },
            ),
            FontSize::new(theme.outline_panel_font_size),
        ))
        .set_parent(outline_entry);

    outline_entry
}

fn add_upgrades_to_outline(
    mut commands: Commands,
    mut events: EventReader<UpgradeEvent>,
    theme: Res<Config>,
    upgrade_list: Res<UpgradeList>,
    container_query: Query<Entity, With<IsOutlineContainer>>,
) {
    let theme = &theme.editor_screen.dark_theme;
    for event in events.read() {
        let upgrade = upgrade_list.get(event.0);

        // Don't add the upgrade if it's repeating.
        if upgrade.remaining == usize::MAX {
            continue;
        }

        // TODO: Denote levels of the upgrades instead of adding them repeatedly.
        // Example: Adding "X" two times should create a single "X (2)" entry instead of
        // two "X" lines.

        for container in &container_query {
            let outline_entry = spawn_outline_entry(&mut commands, theme, upgrade);
            commands.entity(outline_entry).set_parent(container);
        }
    }
}

fn update_outline_header(
    simulation: Res<Simulation>,
    mut info_bar_query: Query<&mut Text, With<IsOutlineHeader>>,
) {
    let info = format!("Installed ({})", simulation.upgrades);

    for mut text in &mut info_bar_query {
        text.sections[0].value = info.clone();
    }
}
