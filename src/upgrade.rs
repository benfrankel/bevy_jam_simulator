use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use strum::EnumCount;

use crate::config::Config;
use crate::simulation::Simulation;
use crate::simulation::SpawnEvent;
use crate::state::editor_screen::spawn_editor_screen;
use crate::state::editor_screen::SceneView;
use crate::state::editor_screen::SceneViewBounds;
use crate::state::editor_screen::UpgradeContainer;
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
                (
                    (enable_upgrades, run_active_upgrades, apply_deferred)
                        .chain()
                        .in_set(AppSet::RunUpgrades),
                    apply_cost_scaling
                        .in_set(AppSet::Simulate)
                        .run_if(on_event::<UpgradeEvent>()),
                ),
            );
    }
}

pub struct Upgrade {
    /// The name of the upgrade. This will be shown on the button.
    pub name: String,
    /// The description of the upgrade. This will be shown as a tooltip.
    pub description: String,

    /// How many lines of code this upgrade costs (will increase with cost scaling)
    pub cost: f64,
    /// The amount by which cost is multiplied whenever an upgrade is enabled.
    pub cost_multiplier: f64,
    /// The relative odds of this upgrade being offered
    pub weight: f32,
    /// How many more copies of this upgrade can be enabled
    pub remaining: usize,

    /// A one-shot system that runs whenever a copy of this upgrade is enabled
    pub enable: Option<SystemId>,
    /// A one-shot system that runs every frame for each active copy of this upgrade
    pub update: Option<SystemId>,
}

impl Default for Upgrade {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            description: "Undefined.".to_string(),
            cost: 0.0,
            cost_multiplier: 1.0,
            weight: 0.0,
            remaining: 1,
            enable: None,
            update: None,
        }
    }
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

fn apply_cost_scaling(mut upgrade_list: ResMut<UpgradeList>) {
    for upgrade in &mut upgrade_list.0 {
        upgrade.cost *= upgrade.cost_multiplier;
    }
}

#[derive(Resource, Default)]
pub struct UpgradeList(pub Vec<Upgrade>);

impl UpgradeList {
    pub fn get(&self, kind: UpgradeKind) -> &Upgrade {
        &self.0[kind as usize]
    }
}

/// The initial sequence of upgrades.
pub const INITIAL_UPGRADES: [UpgradeKind; 5] = [
    UpgradeKind::DarkMode,
    UpgradeKind::TouchOfLifePlugin,
    UpgradeKind::SplashOfLifePlugin,
    UpgradeKind::Brainstorm,
    UpgradeKind::ImportLibrary,
];

/// A macro that generates UpgradeKind enum and load_upgrade_list system from the given
/// UpgradeKind: Upgrade pairs.
macro_rules! generate_upgrade_list {
    (|$world:ident| $($enumname:ident: $upgrade:expr),+ $(,)?) => {
        /// Enum containing all upgrade types.
        #[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash, EnumCount)]
        pub enum UpgradeKind {
            $($enumname),+
        }

        pub const ALL_UPGRADE_KINDS: [UpgradeKind; UpgradeKind::COUNT] = [
            $(UpgradeKind::$enumname),+
        ];

        /// A system that initializes and inserts the UpgradeList resource.
        fn load_upgrade_list($world: &mut World) {
            let upgrade_list = UpgradeList(vec![
                $($upgrade),+
            ]);

            $world.insert_resource(upgrade_list);
        }
    };
}

generate_upgrade_list!(
    |world|
    DarkMode: Upgrade {
        name: "Dark Mode".to_string(),
        description: "Rite of passage for all developers. Required to write code.".to_string(),
        enable: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
            upgrade_list: Res<UpgradeList>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(&mut commands, &config.editor_screen, &upgrade_list, false);
            commands.entity(editor_screen).set_parent(root.ui);
        })),
        ..default()
    },
    TouchOfLifePlugin: Upgrade {
        name: "TouchOfLifePlugin".to_string(),
        description: "Spawns 1 entity whenever you click inside the scene view.".to_string(),
        cost: 5.0,
        enable: Some(
            world.register_system(|mut scene_view_query: Query<&mut SceneView>| {
                for mut scene_view in &mut scene_view_query {
                    scene_view.spawns_per_click += 1;
                }
            }),
        ),
        ..default()
    },
    Brainstorm: Upgrade {
        name: "Brainstorm".to_string(),
        description: "Adds 1 extra upgrade slot.".to_string(),
        enable: Some(
            world.register_system(|mut query: Query<&mut UpgradeContainer>| {
                for mut container in &mut query {
                    container.slots += 1;
                }
            }),
        ),
        ..default()
    },
    SplashOfLifePlugin: Upgrade {
        name: "SplashOfLifePlugin".to_string(),
        description: "Spawns 32 entities immediately.".to_string(),
        cost: 1.0,
        cost_multiplier: 1.2,
        weight: 1.0,
        remaining: usize::MAX,
        enable: Some(
            world.register_system(|mut events: EventWriter<SpawnEvent>, bounds: Res<SceneViewBounds>| {
                for _ in 0..32 {
                    events.send(SpawnEvent((bounds.min.xy() + bounds.max.xy()) / 2.0));
                }
            }),
        ),
        ..default()
    },
    ImportLibrary: Upgrade {
        name: "Import Library".to_string(),
        description: "Writes 10 lines of code immediately.".to_string(),
        cost: 1.0,
        weight: 1.0,
        remaining: usize::MAX,
        enable: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.lines += 10.0;
        })),
        ..default()
    },
);
