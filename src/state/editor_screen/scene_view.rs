use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::simulation::SpawnEvent;
use crate::AppRoot;

pub struct SceneViewPlugin;

impl Plugin for SceneViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SceneView>();
    }
}

#[derive(Component, Reflect, Default)]
pub struct SceneView {
    pub spawns_per_click: usize,
}

pub fn spawn_scene_view(commands: &mut Commands) -> Entity {
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
            On::<Pointer<Down>>::run(click_spawn),
            SceneView::default(),
        ))
        .id();

    scene_view
}

fn click_spawn(
    listener: Listener<Pointer<Down>>,
    mut events: EventWriter<SpawnEvent>,
    root: Res<AppRoot>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    scene_view_query: Query<&SceneView>,
) {
    let (camera, camera_gt) = camera_query.get(root.camera).unwrap();
    let world_pos = camera
        .viewport_to_world(camera_gt, listener.pointer_location.position)
        .unwrap()
        .origin
        .truncate();

    for _ in 0..scene_view_query
        .get(listener.target)
        .unwrap()
        .spawns_per_click
    {
        events.send(SpawnEvent(world_pos));
    }
}
