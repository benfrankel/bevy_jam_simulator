use std::f32::consts::TAU;

use bevy::math::vec2;
use bevy::prelude::*;
use rand::Rng;

use crate::physics::Velocity;
use crate::state::editor_screen::WrapWithinSceneView;
use crate::upgrade::UpgradeEvent;
use crate::util::OverflowDespawnQueue;
use crate::AppRoot;
use crate::AppSet;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnEvent>()
            .register_type::<IsEntityCap>()
            .add_event::<SpawnEvent>()
            .init_resource::<Simulation>()
            .add_systems(Startup, spawn_entity_caps)
            .add_systems(
                Update,
                (
                    count_upgrades.in_set(AppSet::Simulate),
                    spawn_entities.in_set(AppSet::Simulate),
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
    mut entity_cap_query: Query<&mut OverflowDespawnQueue, With<IsEntityCap>>,
    mut simulation: ResMut<Simulation>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        simulation.entities += 1.0;

        let speed = rng.gen_range(10.0..=60.0);
        let angle = rng.gen_range(0.0..=TAU);
        let velocity = (speed * Vec2::from_angle(angle)).extend(-0.01);

        let entity = commands
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
                Velocity(velocity),
                WrapWithinSceneView,
            ))
            .set_parent(root.world)
            .id();

        for mut despawn_queue in &mut entity_cap_query {
            despawn_queue.push(entity);
        }
    }
}

#[derive(Component, Reflect)]
struct IsEntityCap;

const HARD_CAP: usize = 1000;

fn spawn_entity_caps(mut commands: Commands) {
    commands.spawn((
        Name::new("HardEntityCap"),
        OverflowDespawnQueue::new(HARD_CAP),
        IsEntityCap,
    ));
}
