use bevy::prelude::*;

use crate::upgrade::UpgradeEvent;
use crate::upgrade::UpgradeList;
use crate::AppSet;

pub struct CostScalingPlugin;

impl Plugin for CostScalingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CostScaling>()
            .init_resource::<CostScaling>()
            .add_systems(
                Update,
                apply_cost_scaling
                    .in_set(AppSet::Simulate)
                    .run_if(on_event::<UpgradeEvent>()),
            );
    }
}

#[derive(Resource, Reflect)]
struct CostScaling {
    multiplier: f64,
}

impl Default for CostScaling {
    fn default() -> Self {
        Self { multiplier: 1.2 }
    }
}

fn apply_cost_scaling(scale: Res<CostScaling>, mut upgrade_list: ResMut<UpgradeList>) {
    for upgrade in &mut upgrade_list.0 {
        upgrade.cost *= scale.multiplier;
    }
}
