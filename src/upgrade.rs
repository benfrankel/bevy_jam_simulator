use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpgradeList>()
            .add_systems(Startup, load_upgrade_list);
    }
}

pub struct Upgrade {
    pub name: String,
    pub description: String,

    pub base_cost: f64,
    pub weight: f32,
    pub remaining: usize,

    pub enable: Option<SystemId>,
    pub update: Option<SystemId>,
}

#[derive(Resource, Default)]
pub struct UpgradeList(Vec<Upgrade>);

impl UpgradeList {
    pub fn get(&self, kind: UpgradeKind) -> &Upgrade {
        &self.0[kind as usize]
    }
}

#[derive(Reflect, Clone, Copy)]
pub enum UpgradeKind {
    ClickToSpawn,
}

fn load_upgrade_list(mut upgrade_types: ResMut<UpgradeList>) {
    upgrade_types.0.extend([
        // ClickToSpawn
        Upgrade {
            name: "ClickToSpawnPlugin".to_string(),
            description: "Spawns 1 entity whenever you click anywhere in the scene view."
                .to_string(),

            base_cost: 10.0,
            weight: 1.0,
            remaining: 1,

            enable: None,
            update: None,
        },
    ]);
}
