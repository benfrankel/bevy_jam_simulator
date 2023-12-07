use bevy::math::vec2;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::config::Config;
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
use crate::upgrade::ALL_UPGRADE_KINDS;
use crate::upgrade::INITIAL_UPGRADES;
use crate::util::DespawnSet;
use crate::AppSet;

pub struct UpgradePanelPlugin;

impl Plugin for UpgradePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UpgradeContainer>()
            .register_type::<UpgradeButton>()
            .register_type::<UpgradeSequence>()
            .init_resource::<UpgradeSequence>()
            .add_systems(
                Update,
                replace_available_upgrades
                    .in_set(AppSet::Update)
                    .run_if(on_event::<UpgradeEvent>()),
            )
            .add_systems(Update, update_upgrade_button_disabled.in_set(AppSet::End));
    }
}

pub fn spawn_upgrade_panel(
    commands: &mut Commands,
    theme: &EditorScreenTheme,
    upgrade_list: &UpgradeList,
    simulation: &Simulation,
) -> Entity {
    let upgrade_panel = commands
        .spawn((
            Name::new("UpgradePanel"),
            NodeBundle {
                style: Style {
                    min_width: theme.upgrade_panel_width,
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(12.0)),
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
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            }),
            FontSize::new(theme.upgrade_panel_header_font_size),
        ))
        .set_parent(upgrade_panel);

    let upgrade_container = commands
        .spawn((
            Name::new("UpgradeContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            },
            UpgradeContainer { slots: 1 },
        ))
        .set_parent(upgrade_panel)
        .id();

    let upgrade_button = spawn_upgrade_button(
        commands,
        theme,
        upgrade_list,
        INITIAL_UPGRADES[0],
        simulation,
    );
    commands
        .entity(upgrade_button)
        .set_parent(upgrade_container);

    let submit_container = commands
        .spawn((
            Name::new("SubmitContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(130.0),
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
                    width: Val::Percent(100.0),
                    height: theme.upgrade_button_height,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(Val::Px(10.0)),
                    padding: UiRect::vertical(Val::Px(4.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
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
                    &upgrade.description,
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
                            upgrade.tech_debt.abs(),
                        )
                    }
                ),
                side: TooltipSide::Left,
                offset: vec2(-12.0, 0.0),
            },
            On::<Pointer<Click>>::run(
                move |mut events: EventWriter<_>, mut simulation: ResMut<Simulation>| {
                    if simulation.lines >= cost {
                        simulation.lines -= cost;
                        events.send(UpgradeEvent(upgrade_kind));
                    }
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
                // TODO: Format for big numbers
                format!("{} line{}", cost, if cost == 1.0 { "" } else { "s" }),
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

fn spawn_submit_button(commands: &mut Commands, theme: &EditorScreenTheme) -> Entity {
    let submit_button = commands
        .spawn((
            Name::new("SubmitButton"),
            ButtonBundle {
                style: Style {
                    width: Val::Percent(80.0),
                    height: theme.submit_button_height,
                    padding: UiRect::all(Val::Px(10.0)),
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
pub struct UpgradeContainer {
    pub slots: usize,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct UpgradeSequence {
    sequence: Vec<UpgradeKind>,
    next_idx: usize,
}

impl Default for UpgradeSequence {
    fn default() -> Self {
        Self {
            sequence: INITIAL_UPGRADES[1..].to_vec(),
            next_idx: 0,
        }
    }
}

impl UpgradeSequence {
    fn next(
        &mut self,
        upgrade_list: &UpgradeList,
        simulation: &Simulation,
        outline: &UpgradeOutline,
    ) -> Option<UpgradeKind> {
        while self.next_idx < self.sequence.len() {
            self.next_idx += 1;
            let kind = self.sequence[self.next_idx - 1];
            if upgrade_list[kind].is_unlocked(simulation, outline) {
                return Some(kind);
            }
        }
        None
    }
}

fn replace_available_upgrades(
    mut commands: Commands,
    mut despawn: ResMut<DespawnSet>,
    config: Res<Config>,
    upgrade_list: Res<UpgradeList>,
    mut upgrade_sequence: ResMut<UpgradeSequence>,
    simulation: Res<Simulation>,
    outline: Res<UpgradeOutline>,
    container_query: Query<(Entity, &Children, &UpgradeContainer)>,
) {
    let theme = &config.editor_screen.dark_theme;
    for (entity, buttons, container) in &container_query {
        // Despawn old upgrade options
        for &button in buttons {
            despawn.recursive(button);
        }

        let mut add_upgrade = |upgrade_kind: UpgradeKind| {
            let upgrade_button = spawn_upgrade_button(
                &mut commands,
                theme,
                &upgrade_list,
                upgrade_kind,
                &simulation,
            );
            commands.entity(upgrade_button).set_parent(entity);
        };

        // Try to fill slots from the initial sequence of upgrades first
        let mut slots = container.slots;
        while slots > 0 {
            let Some(kind) = upgrade_sequence.next(&upgrade_list, &simulation, &outline) else {
                break;
            };
            add_upgrade(kind);
            slots -= 1;
        }

        // Filter the list of all upgrade kinds into just the ones that are unlocked
        let unlocked_upgrades = ALL_UPGRADE_KINDS
            .into_iter()
            .filter(|&kind| upgrade_list[kind].is_unlocked(&simulation, &outline))
            .collect::<Vec<_>>();

        // Choose the next upgrades for the remaining slots randomly (weighted)
        let mut next_upgrades = unlocked_upgrades
            .choose_multiple_weighted(&mut thread_rng(), slots, |&kind| upgrade_list[kind].weight)
            .unwrap()
            .collect::<Vec<_>>();

        // Sort by name
        // TODO: Sort some other way? Don't sort?
        next_upgrades.sort_by_key(|&&kind| &upgrade_list[kind].name);

        for &kind in next_upgrades {
            add_upgrade(kind);
        }
    }
}
