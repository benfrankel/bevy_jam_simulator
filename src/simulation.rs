use bevy::math::vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::camera::CAMERA_SCALING;
use crate::state::editor_screen::EditorLayoutBounds;
use crate::upgrade::UpgradeEvent;
use crate::AppRoot;
use crate::AppSet;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnEvent>()
            .add_event::<SpawnEvent>()
            .init_resource::<Simulation>()
            .add_systems(
                Update,
                (
                    count_upgrades.in_set(AppSet::Simulate),
                    spawn_entities.in_set(AppSet::Simulate),
                    entity_movement.in_set(AppSet::Simulate),
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct Simulation {
    pub upgrades: usize,
    pub lines: f64,
    pub entities: f64,
}

fn count_upgrades(mut events: EventReader<UpgradeEvent>, mut simulation: ResMut<Simulation>) {
    simulation.upgrades += events.read().count();
}

#[derive(Event, Reflect)]
pub struct SpawnEvent(pub Vec2);

fn spawn_entities(
    mut commands: Commands,
    mut events: EventReader<SpawnEvent>,
    root: Res<AppRoot>,
    mut simulation: ResMut<Simulation>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        simulation.entities += 1.0;

        // Represents the maximum velocity for a single dimension.
        const MAX_VELOCITY: f32 = 60.0;

        commands
            .spawn((
                Name::new("Entity"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::Rgba {
                            red: rng.gen_range(0.0..1.0),
                            green: rng.gen_range(0.0..1.0),
                            blue: rng.gen_range(0.0..1.0),
                            alpha: 1.0,
                        },
                        custom_size: Some(vec2(8.0, 8.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(event.0.extend(0.0)),
                    ..default()
                },
                Velocity(Vec2 {
                    x: rng.gen_range(-MAX_VELOCITY..MAX_VELOCITY),
                    y: rng.gen_range(-MAX_VELOCITY..MAX_VELOCITY),
                }),
            ))
            .set_parent(root.world);
    }
}

#[derive(Component)]
struct Velocity(pub Vec2);

fn entity_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity, &Sprite)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    layout_bounds: Res<EditorLayoutBounds>,
) {
    let window = window_query.single();
    // Subtract the total panel width
    let x_max = (window.resolution.width() / 2.0 - layout_bounds.right) / CAMERA_SCALING;
    let x_min = -(window.resolution.width() / 2.0 - layout_bounds.left) / CAMERA_SCALING;
    let y_max = (window.resolution.height() / 2.0 - layout_bounds.top) / CAMERA_SCALING;
    let y_min = -(window.resolution.height() / 2.0 - layout_bounds.bottom) / CAMERA_SCALING;

    let dt = time.delta_seconds();
    for (mut transform, velocity, sprite) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;

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
