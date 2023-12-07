use bevy::prelude::*;

use crate::AppSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .init_resource::<PhysicsSettings>()
            .add_systems(Update, apply_velocity.in_set(AppSet::Simulate));
    }
}

pub const UNIT_SPEED: f32 = 8.0;

#[derive(Resource, Default)]
pub struct PhysicsSettings {
    pub speed_multiplier: f32,
}

#[derive(Component, Reflect)]
pub struct Velocity(pub Vec3);

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
    physics_settings: Res<PhysicsSettings>,
) {
    let dt = time.delta_seconds();
    let mul = physics_settings.speed_multiplier * dt;
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * mul;
        transform.translation.y += velocity.0.y * mul;
        transform.translation.z += velocity.0.z * dt;
    }
}
