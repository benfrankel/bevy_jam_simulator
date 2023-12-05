use bevy::math::vec2;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_mod_picking::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::state::AppState::*;
use crate::ui::CodeTyper;
use crate::ui::FontSize;
use crate::ui::InteractionColor;
use crate::ui::Tooltip;
use crate::ui::TooltipSide;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;
use crate::AppRoot;

pub struct EditorScreenStatePlugin;

impl Plugin for EditorScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EditorScreenAssets>()
            .init_collection::<EditorScreenAssets>()
            .add_systems(OnEnter(EditorScreen), enter_editor_screen)
            .add_systems(OnExit(EditorScreen), exit_editor_screen)
            .add_systems(Update, update_info_bar_text);
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct EditorScreenConfig {
    info_bar_height: Val,
    info_bar_background_color: Color,
    info_bar_text_color: Color,
    info_bar_font_size: Val,

    plugin_view_width: Val,
    plugin_view_background_color: Color,
    plugin_view_highlight_color: Color,
    plugin_view_text_color: Color,
    plugin_view_font_size: Val,

    scene_view_background_color: Color,

    code_view_height: Val,
    code_view_background_color: Color,
    code_view_text_color: Color,
    code_view_font_size: Val,
    code_view_lines_max: usize,

    upgrade_view_width: Val,
    upgrade_view_background_color: Color,

    upgrade_button_height: Val,
    upgrade_button_normal_color: Color,
    upgrade_button_hovered_color: Color,
    upgrade_button_pressed_color: Color,
    upgrade_button_text_color: Color,
    upgrade_button_font_size: Val,

    submit_button_height: Val,
    submit_button_normal_color: Color,
    submit_button_hovered_color: Color,
    submit_button_pressed_color: Color,
    submit_button_text_color: Color,
    submit_button_font_size: Val,
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EditorScreenAssets {
    // TODO: Music / SFX, sprites
}

fn enter_editor_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    let config = &config.editor_screen;
    commands.insert_resource(ClearColor(config.scene_view_background_color));

    let editor_screen = commands
        .spawn((
            Name::new("EditorScreen"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    let info_bar = commands
        .spawn((
            Name::new("InfoBar"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: config.info_bar_height,
                    padding: UiRect::horizontal(Val::Px(16.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: config.info_bar_background_color.into(),
                ..default()
            },
        ))
        .set_parent(editor_screen)
        .id();

    commands
        .spawn((
            Name::new("InfoBarText"),
            TextBundle::from_section(
                "",
                TextStyle {
                    font: BOLD_FONT_HANDLE,
                    color: config.info_bar_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.info_bar_font_size),
            InfoBarText,
        ))
        .set_parent(info_bar);

    let hbox = commands
        .spawn((
            Name::new("HBox"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(editor_screen)
        .id();

    let plugin_view = commands
        .spawn((
            Name::new("PluginView"),
            NodeBundle {
                style: Style {
                    min_width: config.plugin_view_width,
                    height: Val::Percent(100.0),
                    padding: UiRect::new(Val::Px(12.0), Val::Px(12.0), Val::Px(8.0), Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.plugin_view_background_color.into(),
                ..default()
            },
        ))
        .set_parent(hbox)
        .id();

    // TODO: Replace these dummy plugins
    for plugin_name in ["FooPlugin", "BarPlugin", "QuuxPlugin"] {
        let plugin = commands
            .spawn((
                Name::new("Plugin"),
                NodeBundle {
                    style: Style {
                        margin: UiRect::bottom(Val::Px(1.0)),
                        padding: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                },
                Interaction::default(),
                InteractionColor {
                    normal: Color::NONE,
                    hovered: config.plugin_view_highlight_color,
                    pressed: config.plugin_view_highlight_color,
                },
                Tooltip {
                    text: format!("This is the description for {plugin_name}."),
                    side: TooltipSide::Right,
                    offset: vec2(12.0, 0.0),
                },
            ))
            .set_parent(plugin_view)
            .id();

        commands
            .spawn((
                Name::new("PluginText"),
                TextBundle::from_section(
                    plugin_name,
                    TextStyle {
                        font: FONT_HANDLE,
                        color: config.plugin_view_text_color,
                        ..default()
                    },
                ),
                FontSize::new(config.plugin_view_font_size),
            ))
            .set_parent(plugin);
    }

    // TODO: Add scrollbar to plugin view

    let vbox = commands
        .spawn((
            Name::new("VBox"),
            NodeBundle {
                style: Style {
                    min_width: Val::ZERO,
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(hbox)
        .id();

    commands
        .spawn((
            Name::new("SceneView"),
            NodeBundle {
                style: Style {
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(vbox);

    let code_view = commands
        .spawn((
            Name::new("CodeView"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: config.code_view_height,
                    padding: UiRect::all(Val::VMin(2.0)),
                    ..default()
                },
                background_color: config.code_view_background_color.into(),
                ..default()
            },
        ))
        .set_parent(vbox)
        .id();

    commands
        .spawn((
            Name::new("CodeViewText"),
            TextBundle::from_section(
                "// Start typing to generate lines of code!\n\n",
                TextStyle {
                    font: FONT_HANDLE,
                    color: config.code_view_text_color,
                    ..default()
                },
            ),
            FontSize::new(config.code_view_font_size),
            CodeTyper {
                lines_count: 3,
                lines_max: config.code_view_lines_max,
                ..default()
            },
        ))
        .set_parent(code_view);

    let upgrade_view = commands
        .spawn((
            Name::new("UpgradeView"),
            NodeBundle {
                style: Style {
                    min_width: config.upgrade_view_width,
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.upgrade_view_background_color.into(),
                ..default()
            },
        ))
        .set_parent(hbox)
        .id();

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
        ))
        .set_parent(upgrade_view)
        .id();

    // TODO: Replace these dummy upgrades
    for upgrade_name in ["FooPlugin", "BarPlugin", "QuuxPlugin"] {
        let upgrade = commands
            .spawn((
                Name::new("Upgrade"),
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
                    text: format!("This is the description for {upgrade_name}."),
                    side: TooltipSide::Left,
                    offset: vec2(-12.0, 0.0),
                },
            ))
            .set_parent(upgrade_container)
            .id();

        commands
            .spawn((
                Name::new("UpgradeName"),
                TextBundle::from_section(
                    upgrade_name,
                    TextStyle {
                        font: FONT_HANDLE,
                        color: config.upgrade_button_text_color,
                        ..default()
                    },
                ),
                FontSize::new(config.upgrade_button_font_size),
            ))
            .set_parent(upgrade);

        commands
            .spawn((
                Name::new("UpgradePrice"),
                TextBundle::from_section(
                    "16 lines",
                    TextStyle {
                        font: FONT_HANDLE,
                        color: config.upgrade_button_text_color,
                        ..default()
                    },
                ),
                FontSize::new(config.upgrade_button_font_size),
            ))
            .set_parent(upgrade);
    }

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
        .set_parent(upgrade_view)
        .id();

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
            InteractionColor {
                normal: config.submit_button_normal_color,
                hovered: config.submit_button_hovered_color,
                pressed: config.submit_button_pressed_color,
            },
            On::<Pointer<Click>>::run(|mut next_state: ResMut<NextState<_>>| {
                next_state.set(ResultsScreen);
            }),
        ))
        .set_parent(submit_container)
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
}

fn exit_editor_screen(
    mut commands: Commands,
    root: Res<AppRoot>,
    mut transform_query: Query<&mut Transform>,
) {
    commands.entity(root.ui).despawn_descendants();

    // Reset camera
    let Ok(mut transform) = transform_query.get_mut(root.camera) else {
        return;
    };
    transform.translation = Vec2::ZERO.extend(transform.translation.z);
}

#[derive(Component, Reflect)]
pub struct InfoBarText;

fn update_info_bar_text(
    simulation: Res<Simulation>,
    mut info_bar_query: Query<&mut Text, With<InfoBarText>>,
) {
    // TODO: E.g. Format large numbers like 2,346,834 and then 8.435e22
    let plugins = simulation.plugins;
    let lines = simulation.lines;
    let entities = simulation.entities;

    // TODO: Remove "s" if number is equal to 1
    let info = format!("{plugins} plugins, {lines} lines, {entities} entities");

    for mut text in &mut info_bar_query {
        text.sections[0].value = info.clone();
    }
}
