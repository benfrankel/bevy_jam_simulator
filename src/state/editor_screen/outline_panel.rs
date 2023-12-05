use bevy::math::vec2;
use bevy::prelude::*;

use crate::config::Config;
use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::FontSize;
use crate::ui::InteractionColor;
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
        app.register_type::<IsOutlinePanel>()
            .add_systems(Update, add_upgrades_to_outline);
    }
}

#[derive(Component, Reflect)]
struct IsOutlinePanel;

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
            IsOutlinePanel,
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

    outline_panel
}

fn spawn_outline_entry(
    commands: &mut Commands,
    config: &EditorScreenConfig,
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
            InteractionColor {
                normal: Color::NONE,
                hovered: config.outline_panel_highlight_color,
                pressed: config.outline_panel_highlight_color,
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
                    color: config.outline_panel_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.outline_panel_font_size),
        ))
        .set_parent(outline_entry);

    outline_entry
}

fn add_upgrades_to_outline(
    mut commands: Commands,
    mut events: EventReader<UpgradeEvent>,
    config: Res<Config>,
    upgrade_list: Res<UpgradeList>,
    outline_panel_query: Query<Entity, With<IsOutlinePanel>>,
) {
    let config = &config.editor_screen;
    for event in events.read() {
        println!("A");
        let upgrade = upgrade_list.get(event.0);

        for outline_panel in &outline_panel_query {
            println!("B");
            let outline_entry = spawn_outline_entry(&mut commands, &config, upgrade);
            commands.entity(outline_entry).set_parent(outline_panel);
        }
    }
}
