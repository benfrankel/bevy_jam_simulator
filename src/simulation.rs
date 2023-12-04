use bevy::prelude::*;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Simulation>()
            .init_resource::<Simulation>();
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct Simulation {
    pub plugins: f64,
    pub lines: f64,
    pub entities: f64,
}
