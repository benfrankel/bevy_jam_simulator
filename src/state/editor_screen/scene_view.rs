use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::state::editor_screen::EditorScreenConfig;
use crate::AppRoot;

pub struct SceneViewPlugin;

impl Plugin for SceneViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ClickSpawnEvent>()
            .add_event::<ClickSpawnEvent>();
    }
}

#[derive(Event, Reflect)]
pub struct ClickSpawnEvent(pub Vec2);

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
            On::<Pointer<Click>>::run(click_spawn),
        ))
        .id();

    scene_view
}

fn click_spawn(
    mut events: EventWriter<ClickSpawnEvent>,
    root: Res<AppRoot>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let window = window_query.get(root.window).unwrap();
    let (camera, camera_gt) = camera_query.get(root.camera).unwrap();

    let cursor_pos = window.cursor_position().unwrap();
    let world_pos = camera
        .viewport_to_world(camera_gt, cursor_pos)
        .unwrap()
        .origin
        .truncate();

    events.send(ClickSpawnEvent(world_pos));
}
