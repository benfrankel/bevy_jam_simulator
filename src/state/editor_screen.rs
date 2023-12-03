use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::config::Config;
use crate::state::AppState::*;
use crate::ui::BOLD_FONT_HANDLE;
use crate::AppRoot;

mod code_view;
mod entity_view;
mod systems_view;

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
                    code_view::typing_system,
                    code_view::update_bar,
                    entity_view::update_bar,
                    systems_view::button_color_system,
                )
                    .run_if(in_state(EditorScreen)),
            );
    }
}

const TOP_BAR_TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const TOP_BAR_TEXT_STYLE: TextStyle = TextStyle {
    font: BOLD_FONT_HANDLE,
    font_size: 0.0,
    color: TOP_BAR_TEXT_COLOR,
};
const TOP_BAR_FONT_SIZE: f32 = 8.0;
const TOP_BAR_BACKGROUND_COLOR: Color = Color::rgb(0.165, 0.18, 0.184);

const TOP_BAR_SEPARATOR_COLOR: Color = Color::rgb(0.510, 0.612, 0.769);
const TOP_BAR_SEPARATOR_WIDTH: f32 = 1.5;

// The sum of the following should add up to 100.0.
const CODE_VIEW_WIDTH: f32 = 35.0;
const ENTITY_VIEW_WIDTH: f32 = 40.0;
const SYSTEMS_VIEW_WIDTH: f32 = 25.0;

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EditorScreenAssets {}

fn enter_editor_screen(mut commands: Commands, root: Res<AppRoot>, _config: Res<Config>) {
    commands.insert_resource(ClearColor(Color::BLACK));
    code_view::init(&mut commands, &root);
    entity_view::init(&mut commands, &root);
    systems_view::init(&mut commands, &root);
}

fn exit_editor_screen(root: Res<AppRoot>, mut transform_query: Query<&mut Transform>) {
    let Ok(mut transform) = transform_query.get_mut(root.camera) else {
        return;
    };
    transform.translation = Vec2::ZERO.extend(transform.translation.z);
}
