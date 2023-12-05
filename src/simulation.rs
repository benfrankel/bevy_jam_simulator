use bevy::prelude::*;

use crate::upgrade::UpgradeEvent;
use crate::upgrade::UpgradeList;

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

fn count_plugins(
    mut events: EventReader<UpgradeEvent>,
    mut simulation: ResMut<Simulation>,
    upgrade_list: Res<UpgradeList>,
) {
    simulation.plugins += events
        .read()
        .filter(|event| {
            // Ignore upgrades that can repeat indefinitely.
            upgrade_list.get(event.0).remaining != usize::MAX
        })
        .count();
}
