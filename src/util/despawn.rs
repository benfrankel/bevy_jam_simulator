use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::AppSet;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DespawnSet>()
            .init_resource::<DespawnSet>()
            .add_systems(
                Update,
                (apply_overflow_despawn_queue, apply_despawn_set)
                    .chain()
                    .in_set(AppSet::Despawn),
            );
    }
}

#[derive(Component, Reflect)]
pub struct OverflowDespawnQueue {
    pub entities: VecDeque<Entity>,
    pub cap: usize,
}

impl OverflowDespawnQueue {
    pub fn new(cap: usize) -> Self {
        Self {
            entities: VecDeque::with_capacity(2 * cap),
            cap,
        }
    }

    pub fn push(&mut self, entity: Entity) {
        self.entities.push_back(entity);
    }

    pub fn drain_overflow(&mut self) -> impl '_ + Iterator<Item = Entity> {
        let overflow = self.entities.len().saturating_sub(self.cap);
        self.entities.drain(0..overflow)
    }
}

fn apply_overflow_despawn_queue(
    mut despawn: ResMut<DespawnSet>,
    mut queue_query: Query<&mut OverflowDespawnQueue>,
) {
    for mut queue in &mut queue_query {
        for entity in queue.drain_overflow() {
            despawn.recursive(entity);
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DespawnSet(HashSet<Entity>);

#[allow(dead_code)]
impl DespawnSet {
    // Only supports recursive despawning, because Commands::despawn is a godawful footgun that breaks the hierarchy :)
    pub fn recursive(&mut self, entity: Entity) {
        self.0.insert(entity);
    }
}

fn apply_despawn_set(mut commands: Commands, mut despawn: ResMut<DespawnSet>) {
    for entity in despawn.0.drain() {
        if let Some(entity) = commands.get_entity(entity) {
            entity.despawn_recursive();
        }
        // Silently fail if the entity does not exist anymore.
    }
}
