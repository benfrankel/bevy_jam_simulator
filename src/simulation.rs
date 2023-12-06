use bevy::math::vec2;
use bevy::prelude::*;

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
    for event in events.read() {
        simulation.entities += 1.0;

        commands
            .spawn((
                Name::new("Entity"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(vec2(8.0, 8.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(event.0.extend(0.0)),
                    ..default()
                },
            ))
            .set_parent(root.world);
    }
}
