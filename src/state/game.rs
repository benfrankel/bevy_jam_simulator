use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::config::Config;
use crate::state::AppState::*;
use crate::ui::BOLD_FONT_HANDLE;
use crate::AppRoot;

mod code_view;
use code_view::typing_system;
use code_view::update_code_view_bar;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameAssets>()
            .init_collection::<GameAssets>()
            .add_systems(OnEnter(Game), enter_game)
            .add_systems(OnExit(Game), exit_game)
            .add_systems(
                Update,
                (typing_system, update_code_view_bar).run_if(in_state(Game)),
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

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GameAssets {}

fn enter_game(mut commands: Commands, root: Res<AppRoot>, config: Res<Config>) {
    commands.insert_resource(ClearColor(config.bg_color));
    code_view::init(commands, root);
}

fn exit_game(root: Res<AppRoot>, mut transform_query: Query<&mut Transform>) {
    let Ok(mut transform) = transform_query.get_mut(root.camera) else {
        return;
    };
    transform.translation = Vec2::ZERO.extend(transform.translation.z);
}
