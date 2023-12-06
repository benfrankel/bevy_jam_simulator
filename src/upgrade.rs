use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::state::editor_screen::spawn_editor_screen;
use crate::AppRoot;
use crate::AppSet;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UpgradeEvent>()
            .add_event::<UpgradeEvent>()
            .init_resource::<UpgradeList>()
            .init_resource::<ActiveUpgrades>()
            .add_systems(Startup, load_upgrade_list)
            .add_systems(
                Update,
                (enable_upgrades, run_active_upgrades, apply_deferred)
                    .chain()
                    .in_set(AppSet::RunUpgrades),
            );
    }
}

pub struct Upgrade {
    pub name: String,
    pub description: String,

    // How many lines of code this upgrade costs at 1x cost scaling
    pub base_cost: f64,
    // The relative odds of this upgrade being offered
    pub weight: f32,
    // How many more copies of this upgrade can be enabled
    pub remaining: usize,

    // A one-shot system that runs whenever a copy of this upgrade is enabled
    pub enable: Option<SystemId>,
    // A one-shot system that runs every frame for each active copy of this upgrade
    pub update: Option<SystemId>,

    /// Only present in the initial sequence of upgrades.
    /// Determines the next unlocked upgrade.
    pub next_upgrade: Option<UpgradeKind>,
}

#[derive(Event, Reflect, Clone, Copy)]
pub struct UpgradeEvent(pub UpgradeKind);

fn enable_upgrades(world: &mut World, mut reader: Local<ManualEventReader<UpgradeEvent>>) {
    for event in reader
        .read(world.resource::<Events<_>>())
        .copied()
        .collect::<Vec<_>>()
    {
        let &Upgrade { enable, update, .. } = world.resource::<UpgradeList>().get(event.0);
        if let Some(enable) = enable {
            world.run_system(enable).unwrap();
        }
        if let Some(update) = update {
            world.resource_mut::<ActiveUpgrades>().0.push(update);
        }
    }
}

#[derive(Resource, Default)]
struct ActiveUpgrades(Vec<SystemId>);

fn run_active_upgrades(world: &mut World) {
    #[allow(clippy::unnecessary_to_owned)]
    for update in world.resource::<ActiveUpgrades>().0.to_vec() {
        world.run_system(update).unwrap();
    }
}

#[derive(Resource, Default)]
pub struct UpgradeList {
    pub list: Vec<Upgrade>,
}

impl UpgradeList {
    pub fn get(&self, kind: UpgradeKind) -> &Upgrade {
        &self.list[kind as usize]
    }
}

#[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpgradeKind {
    DarkMode,
    TouchOfLifePlugin,
    BurstOfLifePlugin,
    ImportLibrary,
}

fn load_upgrade_list(world: &mut World) {
    let upgrades = vec![
        Upgrade {
            name: "Dark Mode".to_string(),
            description: "Rite of passage for all developers. Required to write code.".to_string(),

            base_cost: 0.0,
            weight: 0.0,
            remaining: 1,

            enable: Some(world.register_system(enable_dark_mode)),
            update: None,

            next_upgrade: Some(UpgradeKind::TouchOfLifePlugin),
        },
        Upgrade {
            name: "TouchOfLifePlugin".to_string(),
            description: "Spawns 1 entity any time you click in the scene view.".to_string(),

            base_cost: 1.0,
            weight: 0.0,
            remaining: 1,

            enable: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
                simulation.spawns_per_click += 1
            })),
            update: None,

            next_upgrade: None,
        },
        Upgrade {
            name: "BurstOfLifePlugin".to_string(),
            description: "Spawns 10 entities immediately.".to_string(),

            base_cost: 1.0,
            weight: 1.0,
            remaining: usize::MAX,

            enable: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
                simulation.entities += 10.0;
            })),
            update: None,

            next_upgrade: None,
        },
        Upgrade {
            name: "Import Library".to_string(),
            description: "Writes 10 lines of code immediately.".to_string(),

            base_cost: 1.0,
            weight: 1.0,
            remaining: usize::MAX,

            enable: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
                simulation.lines += 10.0;
            })),
            update: None,

            next_upgrade: None,
        },
    ];

    let mut upgrade_types: Mut<UpgradeList> = world.get_resource_mut().unwrap();
    upgrade_types.list = upgrades;
}

fn enable_dark_mode(
    mut commands: Commands,
    root: Res<AppRoot>,
    config: Res<Config>,
    upgrade_list: Res<UpgradeList>,
) {
    commands.entity(root.ui).despawn_descendants();
    let editor_screen =
        spawn_editor_screen(&mut commands, &config.editor_screen, &upgrade_list, false);
    commands.entity(editor_screen).set_parent(root.ui);
}
