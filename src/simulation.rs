use std::f32::consts::TAU;

use bevy::math::vec2;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::physics::Velocity;
use crate::state::editor_screen::SceneViewBounds;
use crate::state::editor_screen::WrapWithinSceneView;
use crate::util::OverflowDespawnQueue;
use crate::AppRoot;
use crate::AppSet;

mod score;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnEvent>()
            .register_type::<IsEntityCap>()
            .add_event::<SpawnEvent>()
            .init_resource::<Simulation>()
            .init_resource::<PassiveCodeGen>()
            .init_resource::<PassiveEntitySpawner>()
            .add_systems(Startup, spawn_entity_caps)
            .add_systems(
                Update,
                (
                    spawn_entities,
                    generate_passive_code,
                    spawn_entities_passively,
                )
                    .in_set(AppSet::Simulate),
            );
    }
}

#[derive(Resource)]
pub struct Simulation {
    pub upgrades: usize,
    pub lines: f64,
    pub entities: f64,
    pub tech_debt: f64,

    /// Fun factor, determines the score.
    pub fun_score: f64,
    /// Presentation factor, determines the score.
    pub presentation_score: f64,

    /// Minimum size for new entities.
    pub entity_size_min: f32,
    /// Maximum size for new entities.
    pub entity_size_max: f32,

    /// List of colors that the new entities can have.
    pub entity_colors: Vec<Color>,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            upgrades: 0,
            lines: 0.0,
            entities: 0.0,
            tech_debt: 0.0,
            fun_score: 0.0,
            presentation_score: 0.0,
            entity_size_min: 8.0,
            entity_size_max: 8.0,
            entity_colors: vec![
                Color::rgba(0.0, 0.0, 0.0, 1.0),
                Color::rgba(1.0, 1.0, 1.0, 1.0),
            ],
        }
    }
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

        let speed = rng.gen_range(0.5..=1.5);
        let angle = rng.gen_range(0.0..=TAU);
        let velocity = (speed * Vec2::from_angle(angle)).extend(-0.01);

        let size = rng.gen_range(simulation.entity_size_min..=simulation.entity_size_max);

        let entity = commands
            .spawn((
                Name::new("Entity"),
                SpriteBundle {
                    sprite: Sprite {
                        color: *simulation.entity_colors.choose(&mut rng).unwrap(),
                        custom_size: Some(vec2(size, size)),
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

const HARD_CAP: usize = 10_000;

fn spawn_entity_caps(mut commands: Commands) {
    commands.spawn((
        Name::new("HardEntityCap"),
        OverflowDespawnQueue::new(HARD_CAP),
        IsEntityCap,
    ));
}

/// Resource for handling passive code generation.
#[derive(Resource)]
pub struct PassiveCodeGen {
    pub timer: Timer,
    pub increase: f64,
}

impl Default for PassiveCodeGen {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            increase: 0.0,
        }
    }
}

/// System for handling passive code generation.
fn generate_passive_code(
    time: Res<Time>,
    mut passive_code_gen: ResMut<PassiveCodeGen>,
    mut simulation: ResMut<Simulation>,
) {
    if passive_code_gen.timer.tick(time.delta()).just_finished() {
        passive_code_gen.timer.reset();
        simulation.lines += passive_code_gen.increase;
    }
}

/// Resource for handling passive entity spawning.
#[derive(Resource)]
pub struct PassiveEntitySpawner {
    pub timer: Timer,
    pub amount: f64,
}

impl Default for PassiveEntitySpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            amount: 0.0,
        }
    }
}

/// System for handling passive entity spawning.
fn spawn_entities_passively(
    time: Res<Time>,
    mut entity_spawner: ResMut<PassiveEntitySpawner>,
    mut events: EventWriter<SpawnEvent>,
    bounds: Res<SceneViewBounds>,
) {
    if entity_spawner.timer.tick(time.delta()).just_finished() {
        entity_spawner.timer.reset();
        // simulation.entities += entity_spawner.amount;
        // TODO:
        // This shouldn't use spawn event directly because the spawn event adds one at a time.
        // When amount is huge due to exponential increase, it will cause floating point problems.
        for _ in 0..(entity_spawner.amount as usize) {
            events.send(SpawnEvent((bounds.min.xy() + bounds.max.xy()) / 2.0));
        }
    }
}
