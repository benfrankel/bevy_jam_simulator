mod code_view;
mod entity_view;
mod system_view;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::config::Config;
use crate::state::AppState::*;
use crate::AppRoot;

pub struct EditorScreenStatePlugin;

impl Plugin for EditorScreenStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EditorScreenAssets>()
            .init_collection::<EditorScreenAssets>()
            .add_systems(OnEnter(EditorScreen), enter_editor_screen)
            .add_systems(OnExit(EditorScreen), exit_editor_screen)
            .add_systems(
                Update,
                (
                    code_view::type_code,
                    code_view::update_bar,
                    entity_view::update_bar,
                    system_view::interact_with_upgrade_buttons,
                )
                    .run_if(in_state(EditorScreen)),
            );
    }
}

#[derive(Default, Reflect, Serialize, Deserialize)]
pub struct EditorScreenConfig {
    top_bar_text_color: Color,
    top_bar_font_size: Val,
    top_bar_background_color: Color,
    top_bar_separator_color: Color,
    top_bar_separator_width: Val,

    code_view_width: Val,
    code_view_background_color: Color,
    entity_view_width: Val,
    entity_view_background_color: Color,
    system_view_width: Val,
    system_view_background_color: Color,

    code_text_color: Color,
    code_font_size: Val,

    upgrade_button_text_color: Color,
    upgrade_button_font_size: Val,
    upgrade_button_normal_color: Color,
    upgrade_button_hovered_color: Color,
    upgrade_button_pressed_color: Color,

    tooltip_background_color: Color,
    tooltip_text_color: Color,
    tooltip_font_size: Val,
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EditorScreenAssets {
    // TODO: Music / SFX, sprites
}

fn enter_editor_screen(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    let config = &config.editor_screen;
    commands.insert_resource(ClearColor(config.entity_view_background_color));

    let code_view = code_view::spawn(&mut commands, config);
    let entity_view = entity_view::spawn(&mut commands, config);
    let system_view = system_view::spawn(&mut commands, config);

    commands
        .spawn((
            Name::new("EditorScreen"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_items: JustifyItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .set_parent(root.ui)
        .push_children(&[code_view, entity_view, system_view]);
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
