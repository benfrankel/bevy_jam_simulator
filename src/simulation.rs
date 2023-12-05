use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Simulation>()
            .add_systems(Update, update_simulation);
    }
}

#[derive(Resource, Default)]
pub struct Simulation {
    pub plugins: Vec<SystemId>,
    pub lines: f64,
    pub entities: f64,
}

fn update_simulation(mut commands: Commands, simulation: Res<Simulation>) {
    for &plugin in &simulation.plugins {
        commands.run_system(plugin);
    }
}
