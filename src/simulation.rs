use bevy::prelude::*;

use crate::upgrade::UpgradeEvent;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Simulation>()
            .add_systems(Update, count_plugins);
    }
}

#[derive(Resource, Default)]
pub struct Simulation {
    pub plugins: usize,
    pub lines: f64,
    pub entities: f64,
}

fn count_plugins(mut events: EventReader<UpgradeEvent>, mut simulation: ResMut<Simulation>) {
    simulation.plugins += events.read().count();
}
