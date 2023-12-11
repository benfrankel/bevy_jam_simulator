use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::simulation::SpawnEvent;
use crate::AppRoot;
use crate::AppSet;

pub struct SceneViewPlugin;

impl Plugin for SceneViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SceneView>()
            .register_type::<SceneViewBounds>()
            .register_type::<WrapWithinSceneView>()
            .init_resource::<SceneViewBounds>()
            .add_systems(
                Update,
                (
                    update_scene_view_bounds.in_set(AppSet::Start),
                    wrap_within_scene_view.in_set(AppSet::End),
                ),
            );
    }
}

#[derive(Component, Reflect, Default)]
pub struct SceneView {
    pub spawns_per_click: f64,
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
    let Ok((camera, camera_gt)) = camera_query.get(root.camera) else {
        return;
    };
    let world_pos = camera
        .viewport_to_world(camera_gt, listener.pointer_location.position)
        .unwrap()
        .origin
        .truncate();

    events.send(SpawnEvent {
        position: world_pos,
        count: scene_view_query
            .get(listener.target)
            .unwrap()
            .spawns_per_click,
    });
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct SceneViewBounds {
    pub min: Vec3,
    pub max: Vec3,
}

fn update_scene_view_bounds(
    root: Res<AppRoot>,
    ui_scale: Res<UiScale>,
    mut bounds: ResMut<SceneViewBounds>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &OrthographicProjection, &GlobalTransform), Without<SceneView>>,
    scene_view_query: Query<(&Node, &GlobalTransform), With<SceneView>>,
) {
    let Ok(window) = window_query.get(root.window) else {
        return;
    };
    let Ok((camera, proj, camera_gt)) = camera_query.get(root.camera) else {
        return;
    };
    let Ok((scene_view, scene_view_gt)) = scene_view_query.get_single() else {
        return;
    };

    let rect = scene_view.physical_rect(scene_view_gt, window.scale_factor(), ui_scale.0);
    let rect = Rect::from_corners(
        camera.viewport_to_world_2d(camera_gt, rect.min).unwrap(),
        camera.viewport_to_world_2d(camera_gt, rect.max).unwrap(),
    );

    let camera_z = camera_gt.translation().z;

    bounds.min = rect.min.extend(camera_z - proj.far);
    bounds.max = rect.max.extend(camera_z - proj.near);
}

#[derive(Component, Reflect, Clone, Copy)]
pub struct WrapWithinSceneView;

fn wrap_within_scene_view(
    bounds: Res<SceneViewBounds>,
    mut wrap_query: Query<(&mut Transform, &TextureAtlasSprite), With<WrapWithinSceneView>>,
) {
    if bounds.min.cmpge(bounds.max).any() {
        return;
    }
    for (mut transform, sprite) in &mut wrap_query {
        let mut min = bounds.min;
        let mut max = bounds.max;

        if let Some(size) = sprite.custom_size {
            min -= (size / 2.0).extend(0.0);
            max += (size / 2.0).extend(0.0);
        }

        let pos = &mut transform.translation;
        *pos = (*pos - min).rem_euclid(max - min) + min;
    }
}
