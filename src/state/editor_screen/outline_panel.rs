use bevy::math::vec2;
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy::utils::HashMap;

use super::ActiveEditorTheme;
use crate::simulation::Simulation;
use crate::state::editor_screen::EditorScreenTheme;
use crate::ui::FontSize;
use crate::ui::InteractionPalette;
use crate::ui::Tooltip;
use crate::ui::TooltipSide;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::upgrade::UpgradeEvent;
use crate::upgrade::UpgradeKind;
use crate::upgrade::UpgradeList;
use crate::AppSet;

pub struct OutlinePanelPlugin;

impl Plugin for OutlinePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsOutlineContainer>()
            .register_type::<IsOutlineHeader>()
            .register_type::<UpgradeOutline>()
            .init_resource::<UpgradeOutline>()
            .add_systems(
                Update,
                (
                    update_outline_container.in_set(AppSet::Update),
                    update_outline_header.in_set(AppSet::Update),
                    update_outline_entry_text
                        .in_set(AppSet::Update)
                        .run_if(resource_changed::<UpgradeOutline>()),
                ),
            );
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct UpgradeOutline(pub HashMap<UpgradeKind, usize>);

// TODO: Add scrollbar
pub fn spawn_outline_panel(commands: &mut Commands, theme: &EditorScreenTheme) -> Entity {
    let outline_panel = commands
        .spawn((
            Name::new("OutlinePanel"),
            NodeBundle {
                style: Style {
                    min_width: theme.outline_panel_width,
                    height: Percent(100.0),
                    padding: UiRect::all(Px(12.0)),
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
                    margin: UiRect::bottom(Px(10.0)),
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
    upgrade_list: &UpgradeList,
    upgrade_kind: UpgradeKind,
) -> Entity {
    let upgrade = &upgrade_list[upgrade_kind];

    let outline_entry = commands
        .spawn((
            Name::new("OutlineEntry"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    margin: UiRect::bottom(Px(1.0)),
                    padding: UiRect::all(Px(4.0)),
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
                text: upgrade.description(),
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
            OutlineEntryText(upgrade_kind),
        ))
        .set_parent(outline_entry);

    outline_entry
}

#[derive(Component, Reflect)]
struct IsOutlineContainer;

fn update_outline_container(
    mut commands: Commands,
    mut events: EventReader<UpgradeEvent>,
    theme: Res<ActiveEditorTheme>,
    upgrade_list: Res<UpgradeList>,
    mut outline: ResMut<UpgradeOutline>,
    container_query: Query<Entity, With<IsOutlineContainer>>,
) {
    let theme = &theme.0;
    for event in events.read() {
        let upgrade_kind = event.0;
        let count = outline.0.entry(upgrade_kind).or_insert(0);
        *count += 1;

        // Don't spawn if marked as no_outline
        // Don't spawn a new outline entry if it's a duplicate
        if upgrade_list[upgrade_kind].no_outline || *count >= 2 {
            continue;
        }

        for container in &container_query {
            let outline_entry =
                spawn_outline_entry(&mut commands, theme, &upgrade_list, upgrade_kind);
            commands.entity(outline_entry).set_parent(container);
        }
    }
}

#[derive(Component, Reflect)]
struct IsOutlineHeader;

fn update_outline_header(
    simulation: Res<Simulation>,
    mut info_bar_query: Query<&mut Text, With<IsOutlineHeader>>,
) {
    let info = format!("Installed ({})", simulation.upgrades);

    for mut text in &mut info_bar_query {
        text.sections[0].value = info.clone();
    }
}

#[derive(Component, Reflect)]
struct OutlineEntryText(UpgradeKind);

fn update_outline_entry_text(
    outline: Res<UpgradeOutline>,
    mut entry_query: Query<(&mut Text, &OutlineEntryText)>,
) {
    for (mut text, entry) in &mut entry_query {
        let count = outline.0[&entry.0];
        if count <= 1 {
            continue;
        }
        let text = &mut text.sections[0].value;

        // Replace the previous count if it exists
        if let Some(idx) = text.find(" (") {
            text.truncate(idx);
        }
        text.push_str(&format!(" ({count})"));
    }
}
