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

pub const CAMERA_WIDTH: f32 = 320.0;
pub const CAMERA_HEIGHT: f32 = 180.0;

fn spawn_camera(mut commands: Commands, mut root: ResMut<AppRoot>) {
    root.camera = commands
        .spawn((
            Name::new("MainCamera"),
            Camera2dBundle {
                projection: OrthographicProjection {
                    near: -1000.0,
                    scaling_mode: ScalingMode::Fixed {
                        width: CAMERA_WIDTH,
                        height: CAMERA_HEIGHT,
                    },
                    ..default()
                },
                ..default()
            },
        ))
        .id();
}
