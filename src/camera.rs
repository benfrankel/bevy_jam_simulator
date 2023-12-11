use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::AppRoot;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off)
            .add_systems(Startup, spawn_camera);
    }
}

#[cfg(not(feature = "web"))]
pub const CAMERA_SCALING: f32 = 4.0;
#[cfg(feature = "web")]
pub const CAMERA_SCALING: f32 = 6.0;

fn spawn_camera(mut commands: Commands, mut root: ResMut<AppRoot>) {
    root.camera = commands
        .spawn((
            Name::new("MainCamera"),
            Camera2dBundle {
                projection: OrthographicProjection {
                    near: -1000.0,
                    scaling_mode: ScalingMode::WindowSize(CAMERA_SCALING),
                    ..default()
                },
                ..default()
            },
        ))
        .id();
}
