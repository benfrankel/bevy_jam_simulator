use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EnableUpgradeEvent>()
            .add_event::<EnableUpgradeEvent>()
            .init_resource::<UpgradeList>()
            .init_resource::<ActiveUpgrades>()
            .add_systems(Startup, load_upgrade_list)
            .add_systems(Update, (enable_upgrades, run_active_upgrades));
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
pub struct ActiveUpgrades(Vec<SystemId>);

fn run_active_upgrades(mut commands: Commands, active_upgrades: Res<ActiveUpgrades>) {
    for &update in &active_upgrades.0 {
        commands.run_system(update);
    }
}

#[derive(Event, Reflect)]
pub struct EnableUpgradeEvent(pub UpgradeKind);

fn enable_upgrades(
    mut commands: Commands,
    mut events: EventReader<EnableUpgradeEvent>,
    upgrade_list: Res<UpgradeList>,
    mut active_upgrades: ResMut<ActiveUpgrades>,
) {
    for event in events.read() {
        let upgrade = upgrade_list.get(event.0);
        if let Some(enable) = upgrade.enable {
            commands.run_system(enable);
        }
        if let Some(update) = upgrade.update {
            active_upgrades.0.push(update);
        }
    }
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
    TouchOfLife,
}

fn load_upgrade_list(mut upgrade_types: ResMut<UpgradeList>) {
    upgrade_types.0.extend([
        // ClickToSpawn
        Upgrade {
            name: "TouchOfLifePlugin".to_string(),
            description: "Spawns 1 entity wherever you click in the scene view.".to_string(),

            base_cost: 10.0,
            weight: 1.0,
            remaining: 1,

            enable: None,
            update: None,
        },
    ]);
}
