use bevy::math::vec2;
use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_mod_picking::prelude::*;

use super::ActiveEditorTheme;
use crate::simulation::Simulation;
use crate::state::editor_screen::EditorScreenTheme;
use crate::state::editor_screen::UpgradeOutline;
use crate::state::AppState;
use crate::ui::Disabled;
use crate::ui::FontSize;
use crate::ui::InteractionPalette;
use crate::ui::Tooltip;
use crate::ui::TooltipSide;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::upgrade::UpgradeEvent;
use crate::upgrade::UpgradeKind;
use crate::upgrade::UpgradeList;
use crate::upgrade::UpgradeSequence;
use crate::util::pretty_num;
use crate::util::DespawnSet;
use crate::AppSet;

pub struct UpgradePanelPlugin;

impl Plugin for UpgradePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsUpgradeContainer>()
            .register_type::<UpgradeButton>()
            .add_systems(
                Update,
                offer_next_upgrades.in_set(AppSet::Update).run_if(
                    on_event::<UpgradeEvent>().or_else(
                        state_changed::<AppState>().and_then(in_state(AppState::EditorScreen)),
                    ),
                ),
            )
            .add_systems(Update, update_upgrade_button_disabled.in_set(AppSet::End));
    }
}

pub fn spawn_upgrade_panel(commands: &mut Commands, theme: &EditorScreenTheme) -> Entity {
    let upgrade_panel = commands
        .spawn((
            Name::new("UpgradePanel"),
            NodeBundle {
                style: Style {
                    min_width: theme.upgrade_panel_width,
                    height: Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: theme.upgrade_panel_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("UpgradeHeader"),
            TextBundle::from_section(
                "Upgrades",
                TextStyle {
                    font: BOLD_FONT_HANDLE,
                    color: theme.upgrade_panel_text_color,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Px(15.0)),
                ..default()
            }),
            FontSize::new(theme.upgrade_panel_header_font_size),
        ))
        .set_parent(upgrade_panel);

    commands
        .spawn((
            Name::new("UpgradeContainer"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            },
            IsUpgradeContainer,
        ))
        .set_parent(upgrade_panel);

    let submit_container = commands
        .spawn((
            Name::new("SubmitContainer"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Px(130.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(upgrade_panel)
        .id();

    let submit_button = spawn_submit_button(commands, theme);
    commands.entity(submit_button).set_parent(submit_container);

    upgrade_panel
}

fn spawn_upgrade_button(
    commands: &mut Commands,
    theme: &EditorScreenTheme,
    upgrade_list: &UpgradeList,
    upgrade_kind: UpgradeKind,
    simulation: &Simulation,
) -> Entity {
    let upgrade = &upgrade_list[upgrade_kind];
    let cost = upgrade.cost(simulation);

    let upgrade_button = commands
        .spawn((
            Name::new("UpgradeButton"),
            ButtonBundle {
                style: Style {
                    width: Percent(100.0),
                    height: theme.upgrade_button_height,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Px(10.0)),
                    padding: UiRect::vertical(Px(4.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(4.0),
                    ..default()
                },
                background_color: theme.upgrade_button_normal_color.into(),
                ..default()
            },
            Disabled(false),
            InteractionPalette {
                normal: theme.upgrade_button_normal_color,
                hovered: theme.upgrade_button_hovered_color,
                pressed: theme.upgrade_button_pressed_color,
                disabled: theme.upgrade_button_disabled_color,
            },
            Tooltip {
                // I have created technical debt to display the technical debt
                text: format!(
                    "{}{}",
                    &upgrade.description(),
                    if upgrade.tech_debt == 0.0 {
                        "".to_string()
                    } else {
                        format!(
                            "\n\n{} technical debt by {}.",
                            if upgrade.tech_debt > 0.0 {
                                "Increases"
                            } else {
                                "Decreases"
                            },
                            (upgrade.tech_debt.abs() * 1000.0).round() / 1000.0,
                        )
                    }
                ),
                side: TooltipSide::Left,
                offset: vec2(-12.0, 0.0),
            },
            On::<Pointer<Click>>::run(
                move |mut events: EventWriter<_>,
                      mut simulation: ResMut<Simulation>,
                      upgrade_list: Res<UpgradeList>| {
                    if simulation.lines < cost {
                        return;
                    }
                    simulation.lines -= cost;

                    let upgrade = &upgrade_list[upgrade_kind];
                    events.send(UpgradeEvent {
                        kind: upgrade_kind,
                        name: upgrade.name.clone(),
                        desc: upgrade.description(),
                    });
                },
            ),
            UpgradeButton(upgrade_kind),
        ))
        .id();

    commands
        .spawn((
            Name::new("UpgradeName"),
            TextBundle::from_section(
                upgrade.name.clone(),
                TextStyle {
                    font: FONT_HANDLE,
                    color: theme.upgrade_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(theme.upgrade_button_font_size),
        ))
        .set_parent(upgrade_button);

    commands
        .spawn((
            Name::new("UpgradeCost"),
            TextBundle::from_section(
                format!(
                    "{} line{}",
                    pretty_num(cost),
                    if cost == 1.0 { "" } else { "s" }
                ),
                TextStyle {
                    font: FONT_HANDLE,
                    color: theme.upgrade_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(theme.upgrade_button_font_size),
        ))
        .set_parent(upgrade_button);

    upgrade_button
}

fn spawn_separator(commands: &mut Commands, theme: &EditorScreenTheme, parent: Entity) {
    commands
        .spawn((
            Name::new("Separator"),
            NodeBundle {
                style: Style {
                    max_height: Px(0.0),
                    border: UiRect::top(theme.separator_width),
                    margin: UiRect {
                        left: Px(0.0),
                        right: Px(0.0),
                        // 16 vertical margin, compensates for buttons
                        top: Px(6.0),
                        bottom: Px(16.0),
                    },
                    ..default()
                },
                border_color: theme.separator_color.into(),
                ..default()
            },
        ))
        .set_parent(parent);
}

fn spawn_submit_button(commands: &mut Commands, theme: &EditorScreenTheme) -> Entity {
    let submit_button = commands
        .spawn((
            Name::new("SubmitButton"),
            ButtonBundle {
                style: Style {
                    width: Percent(80.0),
                    height: theme.submit_button_height,
                    padding: UiRect::all(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: theme.submit_button_normal_color.into(),
                ..default()
            },
            InteractionPalette {
                normal: theme.submit_button_normal_color,
                hovered: theme.submit_button_hovered_color,
                pressed: theme.submit_button_pressed_color,
                disabled: Color::NONE,
            },
            On::<Pointer<Click>>::run(|mut next_state: ResMut<NextState<_>>| {
                next_state.set(AppState::ResultsScreen);
            }),
        ))
        .id();

    commands
        .spawn((
            Name::new("SubmitButtonText"),
            TextBundle::from_section(
                "Submit",
                TextStyle {
                    font: BOLD_FONT_HANDLE,
                    color: theme.submit_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(theme.submit_button_font_size),
        ))
        .set_parent(submit_button);

    submit_button
}

#[derive(Component, Reflect)]
struct UpgradeButton(UpgradeKind);

fn update_upgrade_button_disabled(
    simulation: Res<Simulation>,
    upgrade_list: Res<UpgradeList>,
    mut button_query: Query<(&UpgradeButton, &mut Disabled)>,
) {
    for (button, mut disabled) in &mut button_query {
        disabled.0 = simulation.lines < upgrade_list[button.0].cost(&simulation);
    }
}

#[derive(Component, Reflect)]
struct IsUpgradeContainer;

fn offer_next_upgrades(
    mut commands: Commands,
    mut despawn: ResMut<DespawnSet>,
    theme: Res<ActiveEditorTheme>,
    upgrade_list: Res<UpgradeList>,
    mut sequence: ResMut<UpgradeSequence>,
    simulation: Res<Simulation>,
    outline: Res<UpgradeOutline>,
    container_query: Query<(Entity, Option<&Children>), With<IsUpgradeContainer>>,
) {
    let theme = &theme.0;
    for (entity, buttons) in &container_query {
        // Despawn old upgrade options
        for &button in buttons.into_iter().flatten() {
            despawn.recursive(button);
        }

        let (next_upgrades, desc) = sequence.next(&upgrade_list, &simulation, &outline);

        for kind in next_upgrades {
            if kind == UpgradeKind::RefreshUpgradeList {
                // Add a separator.
                spawn_separator(&mut commands, theme, entity);
            }
            let upgrade_button =
                spawn_upgrade_button(&mut commands, theme, &upgrade_list, kind, &simulation);
            commands.entity(upgrade_button).set_parent(entity);
        }

        // Show description if present.
        if !desc.is_empty() {
            spawn_separator(&mut commands, theme, entity);
            let mut text_bundle = TextBundle::from_section(
                desc,
                TextStyle {
                    font: FONT_HANDLE,
                    color: theme.outline_panel_text_color,
                    ..default()
                },
            );
            // Panel width - horizontal padding
            text_bundle.style.max_width = Px(280.0 - 24.0);
            commands
                .spawn((
                    Name::new("Description"),
                    text_bundle,
                    FontSize::new(theme.outline_panel_font_size),
                ))
                .set_parent(entity);
        }
    }
}
