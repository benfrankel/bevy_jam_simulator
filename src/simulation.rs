mod sprite_pack;

use std::f32::consts::TAU;

use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::physics::Velocity;
pub use crate::simulation::sprite_pack::AtlasList;
pub use crate::simulation::sprite_pack::SkinSet;
pub use crate::simulation::sprite_pack::SpritePack;
pub use crate::simulation::sprite_pack::SpritePackAssets;
pub use crate::simulation::sprite_pack::SpritePackEvent;
use crate::spawn_logical_entities;
use crate::state::editor_screen::SceneViewBounds;
use crate::state::editor_screen::WrapWithinSceneView;
use crate::state::AppState;
use crate::ui::CodeTyper;
use crate::AppRoot;
use crate::AppSet;

mod score;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpawnEvent>()
            .register_type::<EntityPool>()
            .add_plugins(sprite_pack::SpritePackPlugin)
            .add_event::<SpawnEvent>()
            .add_event::<LinesAddedEvent>()
            .init_resource::<EntityPool>()
            .init_resource::<Simulation>()
            .init_resource::<PassiveCodeTyper>()
            .init_resource::<PassiveEntitySpawner>()
            .add_systems(Startup, spawn_entity_pool.after(spawn_logical_entities))
            .add_systems(OnExit(AppState::EditorScreen), reset_entity_pool)
            .add_systems(
                Update,
                (
                    spawn_entities,
                    type_code_passively,
                    spawn_entities_passively,
                    handle_line_added_events,
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
    /// Fun score, affects the submission's results.
    pub fun_score: f64,
    /// Presentation factor, affects the submissions' results.
    pub presentation_score: f64,

    /// Total lines generated during this playthrough before the costs are subtracted.
    pub total_lines: f64,

    /// Newly added line count will be multiplied by this.
    pub line_multiplier: f64,

    /// For each added line, this many entities will spawn.
    pub entity_spawn_per_line: f64,
    /// The count in each SpawnEvent will be multiplied by this.
    pub entity_spawn_multiplier: f64,

    /// Minimum size for new entities.
    pub entity_size_min: f32,
    /// Maximum size for new entities.
    pub entity_size_max: f32,
    /// List of colors that the new entities can have.
    pub entity_colors: Vec<Color>,
    /// The set of entity skins to choose from.
    pub skin_set: SkinSet,

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

            total_lines: 0.0,

            line_multiplier: 1.0,

            entity_spawn_per_line: 0.0,
            entity_spawn_multiplier: 1.0,

            entity_size_min: 8.0,
            entity_size_max: 8.0,
            entity_colors: vec![
                Color::rgba(0.0, 0.0, 0.0, 1.0),
                Color::rgba(1.0, 1.0, 1.0, 1.0),
            ],
            skin_set: default(),

            spawn_offset_min: 0.0,
            spawn_offset_max: 2.0,
        }
    }
}

const ENTITY_CAP: usize = 10_000;

#[derive(Resource, Reflect)]
struct EntityPool {
    entities: Vec<Entity>,
    old_idx: usize,
}

impl Default for EntityPool {
    fn default() -> Self {
        Self {
            entities: Vec::with_capacity(ENTITY_CAP),
            old_idx: 0,
        }
    }
}

impl EntityPool {
    fn recycle(&mut self) -> Entity {
        self.old_idx += 1;
        if self.old_idx > self.entities.len() {
            self.old_idx -= self.entities.len();
        }
        self.entities[self.old_idx - 1]
    }
}

fn spawn_entity_pool(world: &mut World) {
    world.resource_scope(|world: &mut World, mut pool: Mut<EntityPool>| {
        let capacity = pool.entities.capacity() - pool.entities.len();
        pool.entities.extend(
            world.spawn_batch(
                std::iter::repeat((
                    Name::new("Entity"),
                    // NOTE: Workaround for SpatialBundle not impling Clone
                    (
                        Visibility::Hidden,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        Transform::default(),
                        GlobalTransform::default(),
                    ),
                    WrapWithinSceneView,
                    Velocity::default(),
                    TextureAtlasSprite::default(),
                    Handle::<TextureAtlas>::default(),
                ))
                .take(capacity),
            ),
        );

        let parent = world.resource::<AppRoot>().world;
        for &entity in &pool.entities {
            world.entity_mut(entity).set_parent(parent);
        }
    });
}

fn reset_entity_pool(pool: Res<EntityPool>, mut visibility_query: Query<&mut Visibility>) {
    for &entity in &pool.entities {
        let Ok(mut visibility) = visibility_query.get_mut(entity) else {
            continue;
        };
        *visibility = Visibility::Hidden;
    }
}

/// Maximum number of entities that can be spawned in the scene view in a single SpawnEvent.
const MAX_SPAWN_PER_EVENT: usize = 16;

#[derive(Event, Reflect, Clone, Copy)]
pub struct SpawnEvent {
    pub position: Vec2,
    pub count: f64,
}

fn spawn_entities(world: &mut World, mut reader: Local<ManualEventReader<SpawnEvent>>) {
    let mut rng = SmallRng::from_entropy();
    for event in reader
        .read(world.resource::<Events<_>>())
        .copied()
        .collect::<Vec<_>>()
    {
        let mut simulation = world.resource_mut::<Simulation>();
        simulation.entities += event.count * simulation.entity_spawn_multiplier;

        let simulation = world.resource::<Simulation>();
        let spawn_count = MAX_SPAWN_PER_EVENT.min(event.count as usize);
        let mut bundles = vec![];
        for _ in 0..spawn_count {
            let angle = rng.gen_range(0.0..=TAU);
            let direction = Vec2::from_angle(angle);

            let speed = rng.gen_range(0.5..=1.5);
            let velocity = (speed * direction).extend(-0.01);

            let offset = rng.gen_range(simulation.spawn_offset_min..=simulation.spawn_offset_max)
                * direction;
            let position = (event.position + offset).extend(0.0);
            let transform = Transform::from_translation(position);

            let size = rng.gen_range(simulation.entity_size_min..=simulation.entity_size_max);
            let size = Vec2::splat(size);

            let (sprite, texture) =
                simulation
                    .skin_set
                    .bundle(world.resource::<SpritePackAssets>(), size, &mut rng);

            bundles.push((
                Visibility::Inherited,
                transform,
                Velocity(velocity),
                sprite,
                texture,
            ));
        }

        // TODO: Technically, we don't need to set the velocity. We can assign random velocities in spawn_entity_pool
        for (visibility, transform, velocity, sprite, texture) in bundles {
            let entity = world.resource_mut::<EntityPool>().recycle();
            let mut entity = world.entity_mut(entity);
            *entity.get_mut::<Visibility>().unwrap() = visibility;
            *entity.get_mut::<Transform>().unwrap() = transform;
            *entity.get_mut::<Velocity>().unwrap() = velocity;
            *entity.get_mut::<TextureAtlasSprite>().unwrap() = sprite;
            *entity.get_mut::<Handle<TextureAtlas>>().unwrap() = texture;
        }
    }
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
    simulation: Res<Simulation>,
    mut events: EventWriter<LinesAddedEvent>,
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
    let mut new_lines = overflow / typer.overflow_chars_per_line;

    let count = count as usize;
    for (mut code, mut text) in &mut code_query {
        let lines = code.enter(&mut text.sections[0].value, count);
        new_lines += lines;
    }

    events.send(LinesAddedEvent { count: new_lines });
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

#[derive(Event, Reflect)]
pub struct LinesAddedEvent {
    pub count: f64,
}

fn handle_line_added_events(
    mut events: EventReader<LinesAddedEvent>,
    mut simulation: ResMut<Simulation>,
    mut spawn_events: EventWriter<SpawnEvent>,
    bounds: Res<SceneViewBounds>,
) {
    let mut total: f64 = 0.0;
    for event in events.read() {
        total += event.count;
    }
    total *= simulation.line_multiplier;
    simulation.lines += total;
    simulation.total_lines += total;

    // Spawn entities
    let spawned_entities = (total * simulation.entity_spawn_per_line).floor();
    if spawned_entities > 0.0 {
        spawn_events.send(SpawnEvent {
            position: (bounds.min.xy() + bounds.max.xy()) / 2.0,
            count: spawned_entities,
        });
    }
}
