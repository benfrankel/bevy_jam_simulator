use bevy::ecs::system::SystemId;
use bevy::prelude::*;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::state::editor_screen::spawn_code_panel;
use crate::state::editor_screen::EditorScreenUI;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UpgradeEvent>()
            .add_event::<UpgradeEvent>()
            .init_resource::<UpgradeList>()
            .init_resource::<ActiveUpgrades>()
            .add_systems(Startup, load_upgrade_list)
            .add_systems(Update, (enable_upgrades, run_active_upgrades));
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

#[derive(Resource, Default)]
pub struct ActiveUpgrades(Vec<SystemId>);

fn run_active_upgrades(mut commands: Commands, active_upgrades: Res<ActiveUpgrades>) {
    for &update in &active_upgrades.0 {
        commands.run_system(update);
    }
}

#[derive(Event, Reflect)]
pub struct UpgradeEvent(pub UpgradeKind);

fn enable_upgrades(
    mut commands: Commands,
    mut events: EventReader<UpgradeEvent>,
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
pub struct UpgradeList {
    pub list: Vec<Upgrade>,
}

impl UpgradeList {
    pub fn get(&self, kind: UpgradeKind) -> &Upgrade {
        &self.list[kind as usize]
    }
}

#[derive(Reflect, Clone, Copy)]
pub enum UpgradeKind {
    DarkMode,
    TouchOfLife,
    BurstOfLife,
}

fn load_upgrade_list(world: &mut World) {
    let upgrades = vec![
        Upgrade {
            name: "Dark Mode".to_string(),
            description: "Rite of passage for all developers. Required to write code.".to_string(),

            base_cost: 0.0,
            weight: 0.0,
            remaining: 1,

            enable: Some(world.register_system(dark_mode_enable)),
            update: None,

            next_upgrade: Some(UpgradeKind::TouchOfLife),
        },
        Upgrade {
            name: "Touch of Life".to_string(),
            description: "Spawns 1 entity wherever you click in the scene view.".to_string(),

            base_cost: 1.0,
            weight: 0.0,
            remaining: 1,

            // TODO: This is stil no-op
            enable: None,
            update: None,

            next_upgrade: None,
        },
        Upgrade {
            name: "Burst of Life".to_string(),
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
            description: "Spawns 10 lines immediately.".to_string(),

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

fn dark_mode_enable(
    mut commands: Commands,
    config: Res<Config>,
    mut editor_screen_ui: ResMut<EditorScreenUI>,
) {
    commands
        .entity(editor_screen_ui.code_panel)
        .despawn_recursive();
    editor_screen_ui.code_panel = spawn_code_panel(&mut commands, &config.editor_screen);
    commands
        .entity(editor_screen_ui.code_panel)
        .set_parent(editor_screen_ui.vbox);
}
