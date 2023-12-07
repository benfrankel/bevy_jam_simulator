use std::ops::Index;
use std::ops::IndexMut;

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
use crate::state::editor_screen::UpgradeOutline;
use crate::AppRoot;
use crate::AppSet;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UpgradeEvent>()
            .add_event::<UpgradeEvent>()
            .init_resource::<UpgradeList>()
            .init_resource::<UpgradeUpdateSystems>()
            .add_systems(Startup, load_upgrade_list)
            .add_systems(
                Update,
                (install_upgrades, run_installed_upgrades, apply_deferred)
                    .chain()
                    .in_set(AppSet::RunUpgrades),
            );
    }
}

pub struct Upgrade {
    /// The name of the upgrade. This will be shown on the button.
    pub name: String,
    /// The description of the upgrade. This will be shown as a tooltip.
    pub description: String,
    /// How much this upgrade contributes to the Presentation score of your submission.
    pub presentation_score: f32,
    /// How much this upgrade contributes to the Theme Interpretation score of your submission.
    pub theme_score: f32,
    /// How much this upgrade contributes to the Fun score of your submission.
    pub fun_score: f32,

    /// How many lines of code this upgrade costs without tech debt scaling.
    pub base_cost: f64,
    /// The multiplier to the cost of this upgrade per unit of technical debt.
    pub cost_scale_factor: f64,
    /// The amount of technical debt this upgrade adds when you install it.
    pub tech_debt: f64,
    /// The relative odds of this upgrade being offered.
    pub weight: f32,
    /// How many more copies of this upgrade can be installed.
    pub remaining: usize,
    /// The minimum number of entities required for this upgrade to be offered.
    pub entity_min: f64,
    /// The maximum number of entities allowed for this upgrade to be offered.
    pub entity_max: f64,
    /// The minimum number of lines of code required for this upgrade to be offered.
    pub line_min: f64,
    /// The maximum number of lines of code allowed for this upgrade to be offered.
    pub line_max: f64,
    /// The minimum number of installed upgrades required for this upgrade to be offered.
    pub upgrade_min: usize,
    /// The maximum number of installed upgrades allowed for this upgrade to be offered.
    pub upgrade_max: usize,
    /// A list of (upgrade, count) that must be installed for this upgrade to be offered.
    pub requirements: Vec<(UpgradeKind, usize)>,

    /// A one-shot system that runs whenever a copy of this upgrade is installed.
    pub install: Option<SystemId>,
    /// A one-shot system that runs every frame for each installed copy of this upgrade.
    pub update: Option<SystemId>,
}

impl Default for Upgrade {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            description: "Undefined.".to_string(),
            presentation_score: 0.0,
            theme_score: 0.0,
            fun_score: 0.0,

            base_cost: 0.0,
            cost_scale_factor: 1.0,
            tech_debt: 1.0,
            weight: 0.0,
            remaining: 1,
            entity_min: 0.0,
            entity_max: f64::INFINITY,
            line_min: 0.0,
            line_max: f64::INFINITY,
            upgrade_min: 0,
            upgrade_max: usize::MAX,
            requirements: vec![],

            install: None,
            update: None,
        }
    }
}

impl Upgrade {
    pub fn is_unlocked(&self, simulation: &Simulation, outline: &UpgradeOutline) -> bool {
        self.remaining > 0
            && (self.entity_min <= simulation.entities && simulation.entities <= self.entity_max)
            && (self.line_min <= simulation.lines && simulation.lines <= self.line_max)
            && (self.upgrade_min <= simulation.upgrades && simulation.upgrades <= self.upgrade_max)
            && self
                .requirements
                .iter()
                .all(|(kind, count)| outline.0.get(kind).is_some_and(|x| x >= count))
    }

    pub fn cost(&self, simulation: &Simulation) -> f64 {
        (self.base_cost * self.cost_scale_factor.powf(simulation.tech_debt)).floor()
    }
}

#[derive(Event, Reflect, Clone, Copy)]
pub struct UpgradeEvent(pub UpgradeKind);

fn install_upgrades(world: &mut World, mut reader: Local<ManualEventReader<UpgradeEvent>>) {
    for event in reader
        .read(world.resource::<Events<_>>())
        .copied()
        .collect::<Vec<_>>()
    {
        let Upgrade {
            install, update, ..
        } = world.resource::<UpgradeList>()[event.0];
        if let Some(install) = install {
            world.run_system(install).unwrap();
        }
        if let Some(update) = update {
            world.resource_mut::<UpgradeUpdateSystems>().0.push(update);
        }
    }
}

#[derive(Resource, Default)]
struct UpgradeUpdateSystems(Vec<SystemId>);

fn run_installed_upgrades(world: &mut World) {
    #[allow(clippy::unnecessary_to_owned)]
    for update in world.resource::<UpgradeUpdateSystems>().0.to_vec() {
        world.run_system(update).unwrap();
    }
}

#[derive(Resource, Default)]
pub struct UpgradeList(pub Vec<Upgrade>);

impl Index<UpgradeKind> for UpgradeList {
    type Output = Upgrade;

    fn index(&self, index: UpgradeKind) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<UpgradeKind> for UpgradeList {
    fn index_mut(&mut self, index: UpgradeKind) -> &mut Self::Output {
        &mut self.0[index as usize]
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
        tech_debt: 0.0,
        install: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
            upgrade_list: Res<UpgradeList>,
            simulation: Res<Simulation>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(&mut commands, &config.editor_screen, &upgrade_list, &simulation, false);
            commands.entity(editor_screen).set_parent(root.ui);
        })),
        ..default()
    },
    TouchOfLifePlugin: Upgrade {
        name: "TouchOfLifePlugin".to_string(),
        description: "Spawns 1 entity whenever you click inside the scene view.".to_string(),
        base_cost: 5.0,
        install: Some(
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
        tech_debt: 0.0,
        install: Some(
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
        base_cost: 1.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: usize::MAX,
        install: Some(
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
        base_cost: 1.0,
        tech_debt: 0.0,
        weight: 1.0,
        remaining: usize::MAX,
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.lines += 10.0;
        })),
        ..default()
    },
);
