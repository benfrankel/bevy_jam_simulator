use std::f32::consts::TAU;

use bevy::math::vec2;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::physics::Velocity;
use crate::state::editor_screen::SceneViewBounds;
use crate::state::editor_screen::WrapWithinSceneView;
use crate::state::AppState;
use crate::ui::CodeTyper;
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
            .init_resource::<PassiveCodeTyper>()
            .init_resource::<PassiveEntitySpawner>()
            .add_systems(Startup, spawn_entity_caps)
            .add_systems(
                Update,
                (
                    spawn_entities,
                    type_code_passively,
                    spawn_entities_passively,
                )
                    .run_if(in_state(AppState::EditorScreen))
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

    /// Minimum offset distance for entities on spawn.
    pub spawn_offset_min: f32,
    /// Maximum offset distance for entities on spawn.
    pub spawn_offset_max: f32,
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
            spawn_offset_min: 0.0,
            spawn_offset_max: 2.0,
        }
    }
}

/// Maximum number of entities that can be spawned in the scene view for a single SpawnEvent.
const MAX_SPAWN_PER_EVENT: usize = 32;

#[derive(Event, Reflect)]
pub struct SpawnEvent {
    pub position: Vec2,
    pub count: f64,
}

fn spawn_entities(
    mut commands: Commands,
    mut events: EventReader<SpawnEvent>,
    root: Res<AppRoot>,
    mut entity_cap_query: Query<&mut OverflowDespawnQueue, With<IsEntityCap>>,
    mut simulation: ResMut<Simulation>,
) {
    let mut rng = rand::thread_rng();
    for event in events.read() {
        simulation.entities += event.count;

        let spawn_count = MAX_SPAWN_PER_EVENT.min(event.count as usize);
        for _ in 0..spawn_count {
            let angle = rng.gen_range(0.0..=TAU);
            let direction = Vec2::from_angle(angle);

            let speed = rng.gen_range(0.5..=1.5);
            let velocity = (speed * direction).extend(-0.01);

            let offset = rng.gen_range(simulation.spawn_offset_min..=simulation.spawn_offset_max)
                * direction;
            let position = (event.position + offset).extend(0.0);

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
                        transform: Transform::from_translation(position),
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
}

#[derive(Component, Reflect)]
struct IsEntityCap;

const HARD_CAP: usize = 8000;

fn spawn_entity_caps(mut commands: Commands) {
    commands.spawn((
        Name::new("HardEntityCap"),
        OverflowDespawnQueue::new(HARD_CAP),
        IsEntityCap,
    ));
}

/// Resource for handling passive code generation.
#[derive(Resource)]
pub struct PassiveCodeTyper {
    pub timer: Timer,
    pub chars: f64,

    pub llm_timer: Timer,
    pub chars_per_entity: f64,

    pub max_chars_entered: f64,
    pub overflow_chars_per_line: f64,
}

impl Default for PassiveCodeTyper {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            chars: 0.0,

            llm_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            chars_per_entity: 0.0,

            max_chars_entered: 90.0,
            overflow_chars_per_line: 30.0,
        }
    }
}

/// System for handling passive code generation.
fn type_code_passively(
    time: Res<Time>,
    mut typer: ResMut<PassiveCodeTyper>,
    mut simulation: ResMut<Simulation>,
    mut code_query: Query<(&mut CodeTyper, &mut Text)>,
) {
    let mut chars = 0.0;
    if typer.timer.tick(time.delta()).just_finished() {
        typer.timer.reset();
        chars += typer.chars;
    }

    if typer.llm_timer.tick(time.delta()).just_finished() {
        typer.llm_timer.reset();
        chars += typer.chars_per_entity * simulation.entities;
    }

    if chars == 0.0 {
        return;
    }

    let count = chars.min(typer.max_chars_entered);
    let overflow = chars - count;
    let overflow_lines = overflow / typer.overflow_chars_per_line;
    simulation.lines += overflow_lines;

    let count = count as usize;
    for (mut code, mut text) in &mut code_query {
        code.enter(&mut simulation, &mut text.sections[0].value, count);
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
    mut spawner: ResMut<PassiveEntitySpawner>,
    mut events: EventWriter<SpawnEvent>,
    bounds: Res<SceneViewBounds>,
) {
    if !spawner.timer.tick(time.delta()).just_finished() {
        return;
    }
    spawner.timer.reset();

    events.send(SpawnEvent {
        position: (bounds.min.xy() + bounds.max.xy()) / 2.0,
        count: spawner.amount,
    });
}
