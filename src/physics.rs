use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::camera::CAMERA_SCALING;
use crate::state::editor_screen::EditorLayoutBounds;
use crate::AppSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .register_type::<WrapWithinSceneView>()
            .add_systems(
                Update,
                (
                    apply_velocity.in_set(AppSet::Simulate),
                    wrap_within_scene_view.in_set(AppSet::Simulate),
                ),
            );
    }
}

#[derive(Component, Reflect)]
pub struct Velocity(pub Vec2);

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_seconds();
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

#[derive(Component, Reflect)]
pub struct WrapWithinSceneView;

fn wrap_within_scene_view(
    mut query: Query<(&mut Transform, &Sprite), With<WrapWithinSceneView>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    bounds: Res<EditorLayoutBounds>,
) {
    let window = window_query.single();
    // Subtract the total panel width
    let x_max = (window.resolution.width() / 2.0 - bounds.0.max.x) / CAMERA_SCALING;
    let x_min = -(window.resolution.width() / 2.0 - bounds.0.min.x) / CAMERA_SCALING;
    let y_max = (window.resolution.height() / 2.0 - bounds.0.max.y) / CAMERA_SCALING;
    let y_min = -(window.resolution.height() / 2.0 - bounds.0.min.y) / CAMERA_SCALING;

    for (mut transform, sprite) in &mut query {
        if let Some(size) = sprite.custom_size {
            let x_max = x_max + (size.x / 2.0);
            let x_min = x_min - (size.x / 2.0);
            if transform.translation.x >= x_max {
                transform.translation.x = x_min;
            } else if transform.translation.x <= x_min {
                transform.translation.x = x_max;
            }

            let y_max = y_max + (size.y / 2.0);
            let y_min = y_min - (size.y / 2.0);
            if transform.translation.y >= y_max {
                transform.translation.y = y_min;
            } else if transform.translation.y <= y_min {
                transform.translation.y = y_max;
            }
        }
    }
}
