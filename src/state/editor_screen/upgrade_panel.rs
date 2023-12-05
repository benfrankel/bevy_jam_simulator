use bevy::math::vec2;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::state::editor_screen::EditorScreenConfig;
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

pub struct UpgradePanelPlugin;

impl Plugin for UpgradePanelPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IsUpgradeContainer>()
            .register_type::<UpgradeButton>()
            .add_systems(
                Update,
                (
                    update_upgrade_button_disabled,
                    replace_available_upgrades.run_if(on_event::<UpgradeEvent>()),
                ),
            );
    }
}

const FIRST_UPGRADE: UpgradeKind = UpgradeKind::TouchOfLife;

#[derive(Component, Reflect)]
pub struct IsUpgradeContainer;

#[derive(Component, Reflect)]
struct UpgradeButton(UpgradeKind);

pub fn spawn_upgrade_panel(
    commands: &mut Commands,
    config: &EditorScreenConfig,
    upgrade_list: &UpgradeList,
) -> Entity {
    let upgrade_panel = commands
        .spawn((
            Name::new("UpgradePanel"),
            NodeBundle {
                style: Style {
                    min_width: config.upgrade_panel_width,
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.upgrade_panel_background_color.into(),
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
                    color: config.upgrade_panel_text_color,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            }),
            FontSize::new(config.upgrade_panel_header_font_size),
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
            IsUpgradeContainer,
        ))
        .set_parent(upgrade_panel)
        .id();

    let upgrade_button = spawn_upgrade_button(commands, config, upgrade_list, FIRST_UPGRADE);
    commands
        .entity(upgrade_button)
        .set_parent(upgrade_container);

    let submit_container = commands
        .spawn((
            Name::new("SubmitContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(upgrade_panel)
        .id();

    let submit_button = spawn_submit_button(commands, config);
    commands.entity(submit_button).set_parent(submit_container);

    upgrade_panel
}

// TODO: On EnableUpgradeEvent:
// -> add an entry to outline panel
// -> replace all entries in upgrade panel
fn spawn_upgrade_button(
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
            Disabled(false),
            InteractionPalette {
                normal: config.upgrade_button_normal_color,
                hovered: config.upgrade_button_hovered_color,
                pressed: config.upgrade_button_pressed_color,
                disabled: config.upgrade_button_disabled_color,
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

fn spawn_submit_button(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let submit_button = commands
        .spawn((
            Name::new("SubmitButton"),
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: config.submit_button_height,
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: config.submit_button_normal_color.into(),
                ..default()
            },
            InteractionPalette {
                normal: config.submit_button_normal_color,
                hovered: config.submit_button_hovered_color,
                pressed: config.submit_button_pressed_color,
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
                    color: config.submit_button_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.submit_button_font_size),
        ))
        .set_parent(submit_button);

    submit_button
}

fn update_upgrade_button_disabled(
    simulation: Res<Simulation>,
    upgrade_list: Res<UpgradeList>,
    mut button_query: Query<(&UpgradeButton, &mut Disabled)>,
) {
    for (button, mut disabled) in &mut button_query {
        let upgrade = upgrade_list.get(button.0);
        // TODO: Cost scaling
        disabled.0 = simulation.lines < upgrade.base_cost;
    }
}

fn replace_available_upgrades(
    mut commands: Commands,
    config: Res<Config>,
    container_query: Query<Entity, With<IsUpgradeContainer>>,
) {
    let _config = &config.editor_screen;
    for container in &container_query {
        commands.entity(container).despawn_descendants();

        // TODO: Spawn the next set of upgrades
        // - # of upgrade slots
        // - Initial fixed sequence of upgrades
        // - Randomly chosen upgrades (weighted)
    }
}
