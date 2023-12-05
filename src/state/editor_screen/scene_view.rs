use bevy::prelude::*;

use crate::state::editor_screen::EditorScreenConfig;

pub fn spawn_scene_view(commands: &mut Commands, _config: &EditorScreenConfig) -> Entity {
    let scene_view = commands
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
        .id();

    scene_view
}
