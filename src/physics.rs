use bevy::prelude::*;

use crate::AppSet;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Velocity>()
            .add_systems(Update, apply_velocity.in_set(AppSet::Simulate));
    }
}

#[derive(Component, Reflect)]
pub struct Velocity(pub Vec3);

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let dt = time.delta_seconds();
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * dt;
    }
}
