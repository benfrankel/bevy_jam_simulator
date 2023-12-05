mod code_panel;
mod info_bar;
mod outline_panel;
mod scene_view;
mod upgrade_panel;

// Expose this for the upgrades.
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
pub use code_panel::spawn_code_panel;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::state::editor_screen::code_panel::spawn_light_code_panel;
use crate::state::editor_screen::info_bar::spawn_info_bar;
use crate::state::editor_screen::outline_panel::spawn_outline_panel;
use crate::state::editor_screen::scene_view::spawn_scene_view;
use crate::state::editor_screen::upgrade_panel::spawn_upgrade_panel;
use crate::state::AppState::*;
use crate::upgrade::UpgradeList;
use crate::AppRoot;

pub struct EditorScreenStatePlugin;

impl Plugin for EditorScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EditorScreenAssets>()
            .init_collection::<EditorScreenAssets>()
            .add_systems(OnEnter(EditorScreen), enter_editor_screen)
            .add_systems(OnExit(EditorScreen), exit_editor_screen)
            .add_plugins((
                info_bar::InfoBarPlugin,
                outline_panel::OutlinePanelPlugin,
                upgrade_panel::UpgradePanelPlugin,
            ));
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct EditorScreenConfig {
    info_bar_height: Val,
    info_bar_background_color: Color,
    info_bar_text_color: Color,
    info_bar_font_size: Val,

    outline_panel_width: Val,
    outline_panel_background_color: Color,
    outline_panel_highlight_color: Color,
    outline_panel_text_color: Color,
    outline_panel_font_size: Val,
    outline_panel_header_font_size: Val,

    scene_view_background_color: Color,

    light_theme_background_color: Color,
    light_theme_text_color: Color,

    code_panel_height: Val,
    code_panel_background_color: Color,
    code_panel_text_color: Color,
    code_panel_font_size: Val,
    code_panel_lines_max: usize,

    upgrade_panel_width: Val,
    upgrade_panel_background_color: Color,
    upgrade_panel_text_color: Color,
    upgrade_panel_header_font_size: Val,

    upgrade_button_height: Val,
    upgrade_button_normal_color: Color,
    upgrade_button_hovered_color: Color,
    upgrade_button_pressed_color: Color,
    upgrade_button_disabled_color: Color,
    upgrade_button_text_color: Color,
    upgrade_button_font_size: Val,

    submit_button_height: Val,
    submit_button_normal_color: Color,
    submit_button_hovered_color: Color,
    submit_button_pressed_color: Color,
    submit_button_text_color: Color,
    submit_button_font_size: Val,
}

#[derive(Resource)]
pub struct EditorScreenUI {
    pub vbox: Entity,
    pub code_panel: Entity,
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EditorScreenAssets {
    // TODO: Music / SFX, sprites
}

fn enter_editor_screen(
    mut commands: Commands,
    root: Res<AppRoot>,
    config: Res<Config>,
    upgrade_list: Res<UpgradeList>,
) {
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

    let info_bar = spawn_info_bar(&mut commands, config);
    commands.entity(info_bar).set_parent(editor_screen);

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

    let outline_panel = spawn_outline_panel(&mut commands, config);
    commands.entity(outline_panel).set_parent(hbox);

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

    let scene_view = spawn_scene_view(&mut commands, config);
    commands.entity(scene_view).set_parent(vbox);

    let code_panel = spawn_light_code_panel(&mut commands, config);
    commands.entity(code_panel).set_parent(vbox);

    let upgrade_panel = spawn_upgrade_panel(&mut commands, config, &upgrade_list);
    commands.entity(upgrade_panel).set_parent(hbox);

    commands.insert_resource(EditorScreenUI { vbox, code_panel });
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
