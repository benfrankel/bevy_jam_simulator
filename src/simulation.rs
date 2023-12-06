use bevy::math::vec2;
use bevy::prelude::*;

use crate::state::editor_screen::ClickSpawnEvent;
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
                    spawn_from_click.in_set(AppSet::Simulate),
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct Simulation {
    pub upgrades: usize,
    pub lines: f64,
    pub entities: f64,

    pub spawns_per_click: usize,
}

fn count_upgrades(mut events: EventReader<UpgradeEvent>, mut simulation: ResMut<Simulation>) {
    simulation.upgrades += events.read().count();
}

#[derive(Event, Reflect)]
pub struct SpawnEvent(pub Entity);

fn spawn_from_click(
    mut commands: Commands,
    mut click_events: EventReader<ClickSpawnEvent>,
    mut spawn_events: EventWriter<SpawnEvent>,
    root: Res<AppRoot>,
    mut simulation: ResMut<Simulation>,
) {
    for click_event in click_events.read() {
        simulation.entities += simulation.spawns_per_click as f64;
        for _ in 0..simulation.spawns_per_click {
            let entity = commands
                .spawn((
                    Name::new("Entity"),
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            custom_size: Some(vec2(16.0, 16.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(click_event.0.extend(0.0)),
                        ..default()
                    },
                ))
                .set_parent(root.world)
                .id();
            spawn_events.send(SpawnEvent(entity))
        }
    }
}
