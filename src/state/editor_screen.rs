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
pub use crate::state::editor_screen::outline_panel::UpgradeOutline;
use crate::state::editor_screen::scene_view::spawn_scene_view;
pub use crate::state::editor_screen::scene_view::SceneView;
pub use crate::state::editor_screen::scene_view::SceneViewBounds;
pub use crate::state::editor_screen::scene_view::WrapWithinSceneView;
use crate::state::editor_screen::upgrade_panel::spawn_upgrade_panel;
pub use crate::state::editor_screen::upgrade_panel::UpgradeContainer;
use crate::state::AppState::*;
use crate::upgrade::UpgradeList;
use crate::AppRoot;

pub struct EditorScreenStatePlugin;

impl Plugin for EditorScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EditorScreenAssets>()
            .register_type::<EditorLayoutBounds>()
            .init_resource::<EditorLayoutBounds>()
            .init_collection::<EditorScreenAssets>()
            .add_systems(OnEnter(EditorScreen), enter_editor_screen)
            .add_systems(OnExit(EditorScreen), exit_editor_screen)
            .add_plugins((
                info_bar::InfoBarPlugin,
                outline_panel::OutlinePanelPlugin,
                scene_view::SceneViewPlugin,
                upgrade_panel::UpgradePanelPlugin,
            ));
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct EditorScreenTheme {
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

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct EditorScreenConfig {
    scene_view_background_color: Color,

    light_theme: EditorScreenTheme,
    dark_theme: EditorScreenTheme,
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EditorScreenAssets {
    // TODO: Music / SFX, sprites
}

/// Contains the boundary information for the editor.
#[derive(Resource, Reflect, Default)]
pub struct EditorLayoutBounds {
    /// Padding from the top edge.
    pub top: f32,
    /// Padding from the bottom edge.
    pub bottom: f32,
    /// Padding from the left edge.
    pub left: f32,
    /// Padding from the right edge.
    pub right: f32,
}

fn enter_editor_screen(
    mut commands: Commands,
    root: Res<AppRoot>,
    config: Res<Config>,
    upgrade_list: Res<UpgradeList>,
) {
    let config = &config.editor_screen;
    commands.insert_resource(ClearColor(config.scene_view_background_color));

    let editor_screen = spawn_editor_screen(&mut commands, config, &upgrade_list, true);
    commands.entity(editor_screen).set_parent(root.ui);
}

pub fn spawn_editor_screen(
    commands: &mut Commands,
    config: &EditorScreenConfig,
    upgrade_list: &Res<UpgradeList>,
    light_theme: bool,
) -> Entity {
    let theme = if light_theme {
        &config.light_theme
    } else {
        &config.dark_theme
    };
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
        .id();

    let info_bar = spawn_info_bar(commands, theme);
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

    let outline_panel = spawn_outline_panel(commands, theme);
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

    let scene_view = spawn_scene_view(commands);
    commands.entity(scene_view).set_parent(vbox);

    let code_panel = if light_theme {
        spawn_light_code_panel(commands, theme)
    } else {
        spawn_code_panel(commands, theme)
    };
    commands.entity(code_panel).set_parent(vbox);

    let upgrade_panel = spawn_upgrade_panel(commands, theme, upgrade_list);
    commands.entity(upgrade_panel).set_parent(hbox);

    // Note that insert_resource overwrites if the resource already exists.
    // This could probably use Val::resolve instead of requiring Px, but that's ok.
    commands.insert_resource(EditorLayoutBounds {
        top: match theme.info_bar_height {
            Val::Px(px) => px,
            _ => panic!("info_bar_height must be defined in Px"),
        },
        bottom: match theme.code_panel_height {
            Val::Px(px) => px,
            _ => panic!("code_panel_height must be defined in Px"),
        },
        left: match theme.outline_panel_width {
            Val::Px(px) => px,
            _ => panic!("outline_panel_width must be defined in Px"),
        },
        right: match theme.upgrade_panel_width {
            Val::Px(px) => px,
            _ => panic!("upgrade_panel_width must be defined in Px"),
        },
    });

    editor_screen
}

fn exit_editor_screen(
    mut commands: Commands,
    root: Res<AppRoot>,
    mut transform_query: Query<&mut Transform>,
) {
    commands.insert_resource(EditorLayoutBounds::default());
    commands.entity(root.ui).despawn_descendants();
    commands.entity(root.world).despawn_descendants();

    // Reset camera
    let Ok(mut transform) = transform_query.get_mut(root.camera) else {
        return;
    };
    transform.translation = Vec2::ZERO.extend(transform.translation.z);
}
