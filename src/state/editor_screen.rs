//mod code_view;
//mod entity_view;
//mod system_view;

use bevy::math::vec2;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::state::AppState::*;
use crate::ui::CodeTyper;
use crate::ui::FontSize;
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
            .add_systems(OnExit(EditorScreen), exit_editor_screen);
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

    upgrade_button_normal_color: Color,
    upgrade_button_hovered_color: Color,
    upgrade_button_pressed_color: Color,
    upgrade_button_text_color: Color,
    upgrade_button_font_size: Val,

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
                    height: config.info_bar_height,
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
                    width: config.plugin_view_width,
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

    // TODO: Remove these dummy plugins
    for plugin_name in ["FooPlugin", "BarPlugin", "QuuxPlugin"] {
        let plugin = commands
            .spawn((
                Name::new("Plugin"),
                NodeBundle {
                    style: Style {
                        padding: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                },
                Tooltip {
                    text: format!("This is the description for {plugin_name}."),
                    side: TooltipSide::Right,
                    offset: vec2(12.0, 0.0),
                },
                Interaction::default(),
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

    let vbox = commands
        .spawn((
            Name::new("VBox"),
            NodeBundle {
                style: Style {
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
                    height: config.code_view_height,
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

    let _upgrade_view = commands
        .spawn((
            Name::new("UpgradeView"),
            NodeBundle {
                style: Style {
                    width: config.upgrade_view_width,
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: config.upgrade_view_background_color.into(),
                ..default()
            },
        ))
        .set_parent(hbox)
        .id();

    // TODO: Upgrade view's children
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
