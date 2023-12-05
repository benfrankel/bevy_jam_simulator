use bevy::math::vec2;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::simulation::Simulation;
use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::FontSize;
use crate::ui::InteractionColor;
use crate::ui::Tooltip;
use crate::ui::TooltipSide;
use crate::ui::FONT_HANDLE;
use crate::upgrade::EnableUpgradeEvent;
use crate::upgrade::UpgradeKind;
use crate::upgrade::UpgradeList;

// TODO: On EnableUpgradeEvent:
// -> add an entry to outline panel
// -> replace all entries in upgrade panel
pub fn spawn_upgrade_button(
    commands: &mut Commands,
    config: &EditorScreenConfig,
    upgrade_list: &UpgradeList,
    upgrade_kind: UpgradeKind,
) -> Entity {
    let upgrade = upgrade_list.get(upgrade_kind);
    // TODO: Cost scaling
    let upgrade_cost = upgrade.base_cost;

    let upgrade_button = commands
        .spawn((
            Name::new("UpgradeButton"),
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: config.upgrade_button_height,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    padding: UiRect::vertical(Val::Px(4.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                background_color: config.upgrade_button_normal_color.into(),
                ..default()
            },
            InteractionColor {
                normal: config.upgrade_button_normal_color,
                hovered: config.upgrade_button_hovered_color,
                pressed: config.upgrade_button_pressed_color,
            },
            Tooltip {
                text: upgrade.description.clone(),
                side: TooltipSide::Left,
                offset: vec2(-12.0, 0.0),
            },
            On::<Pointer<Click>>::run(
                move |mut events: EventWriter<_>, mut simulation: ResMut<Simulation>| {
                    if simulation.lines >= upgrade_cost {
                        simulation.lines -= upgrade_cost;
                        events.send(EnableUpgradeEvent(upgrade_kind));
                    }
                },
            ),
        ))
        .id();

    commands
        .spawn((
            Name::new("UpgradeName"),
            TextBundle::from_section(
                upgrade.name.clone(),
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.upgrade_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.upgrade_button_font_size),
        ))
        .set_parent(upgrade_button);

    commands
        .spawn((
            Name::new("UpgradeCost"),
            TextBundle::from_section(
                // TODO: Format for big numbers
                format!("{} lines", upgrade_cost),
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.upgrade_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.upgrade_button_font_size),
        ))
        .set_parent(upgrade_button);

    upgrade_button
}
