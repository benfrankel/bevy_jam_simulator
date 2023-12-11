use bevy::prelude::*;

use crate::AppRoot;
use crate::AppSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .register_type::<PhysicsSettings>()
            .init_resource::<PhysicsSettings>()
            .add_systems(
                Update,
                (apply_mouse_force, apply_velocity)
                    .chain()
                    .in_set(AppSet::Simulate),
            );
    }
}

pub const UNIT_SPEED: f32 = 10.0;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct PhysicsSettings {
    pub speed_multiplier: f32,
    pub mouse_force_strength: f32,
}

#[derive(Component, Reflect, Default, Clone, Copy)]
pub struct Velocity(pub Vec3);

fn apply_velocity(
    time: Res<Time>,
    physics: Res<PhysicsSettings>,
    mut velocity_query: Query<(&mut Transform, &Velocity)>,
) {
    let dt = time.delta_seconds();
    let mul = physics.speed_multiplier * dt;
    for (mut transform, velocity) in &mut velocity_query {
        transform.translation.x += velocity.0.x * mul;
        transform.translation.y += velocity.0.y * mul;
        transform.translation.z += velocity.0.z * dt;
    }
}

fn apply_mouse_force(
    time: Res<Time>,
    physics: Res<PhysicsSettings>,
    root: Res<AppRoot>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut velocity_query: Query<(&mut Velocity, &GlobalTransform), Without<Camera>>,
) {
    let Ok(window) = window_query.get(root.window) else {
        return;
    };
    let Ok((camera, camera_gt)) = camera_query.get(root.camera) else {
        return;
    };
    let Some(cursor_pos) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_gt, cursor))
    else {
        return;
    };

    const MIN_DISTANCE_SQ: f32 = 0.01;
    const MAX_DISTANCE_SQ: f32 = 400.0;

    let dt = time.delta_seconds();
    for (mut velocity, gt) in &mut velocity_query {
        let pos = gt.translation().xy();
        let edge = pos - cursor_pos;
        let distance_sq = edge.length_squared();
        if !(MIN_DISTANCE_SQ..MAX_DISTANCE_SQ).contains(&distance_sq) {
            continue;
        }
        let push = edge / distance_sq;

        let mut v = velocity.0.xy() + push * physics.mouse_force_strength * dt;
        if v.length() < MIN_DISTANCE_SQ {
            v = Vec2::X * MIN_DISTANCE_SQ;
        }
        velocity.0 = v.clamp_length(0.5, 1.5).extend(velocity.0.z);
    }
}
