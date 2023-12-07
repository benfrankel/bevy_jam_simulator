use std::ops::Index;
use std::ops::IndexMut;
use std::time::Duration;

use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use rand::Rng;
use strum::EnumCount;

use crate::config::Config;
use crate::physics::PhysicsSettings;
use crate::physics::UNIT_SPEED;
use crate::simulation::PassiveCodeGen;
use crate::simulation::PassiveEntitySpawner;
use crate::simulation::Simulation;
use crate::simulation::SpawnEvent;
use crate::state::editor_screen::spawn_editor_screen;
use crate::state::editor_screen::SceneView;
use crate::state::editor_screen::SceneViewBounds;
use crate::state::editor_screen::UpgradeContainer;
use crate::state::editor_screen::UpgradeOutline;
use crate::ui::CodeTyper;
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
                (
                    process_new_installed_upgrades,
                    install_upgrades.run_if(on_event::<UpgradeEvent>()),
                    run_installed_upgrades,
                    apply_deferred,
                )
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
    pub presentation_score: f64,
    /// How much this upgrade contributes to the Fun score of your submission.
    pub fun_score: f64,

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
    /// The minimum amount of technical debt required for this upgrade to be offered.
    pub tech_debt_min: f64,
    /// The maximum amount of technical debt allowed for this upgrade to be offered.
    pub tech_debt_max: f64,
    /// A list of (upgrade, count) that must be installed for this upgrade to be offered.
    pub requirements: Vec<(UpgradeKind, usize)>,

    /// A one-shot system that runs whenever any upgrade is installed.
    pub update: Option<SystemId>,
    /// A one-shot system that runs whenever a copy of this upgrade is installed.
    pub install: Option<SystemId>,
    /// A one-shot system that runs every frame for each installed copy of this upgrade.
    pub run: Option<SystemId>,
}

impl Default for Upgrade {
    fn default() -> Self {
        Self {
            name: "Unnamed".to_string(),
            description: "Undefined.".to_string(),
            presentation_score: 0.0,
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
            tech_debt_min: f64::NEG_INFINITY,
            tech_debt_max: f64::INFINITY,
            requirements: vec![],

            update: None,
            install: None,
            run: None,
        }
    }
}

impl Upgrade {
    pub fn is_unlocked(&self, simulation: &Simulation, outline: &UpgradeOutline) -> bool {
        self.remaining > 0
            && (self.entity_min <= simulation.entities && simulation.entities <= self.entity_max)
            && (self.line_min <= simulation.lines && simulation.lines <= self.line_max)
            && (self.upgrade_min <= simulation.upgrades && simulation.upgrades <= self.upgrade_max)
            && (self.tech_debt_min <= simulation.tech_debt
                && simulation.tech_debt <= self.tech_debt_max)
            && self
                .requirements
                .iter()
                .all(|(kind, count)| outline.0.get(kind).is_some_and(|x| x >= count))
    }

    pub fn cost(&self, simulation: &Simulation) -> f64 {
        (self.base_cost * self.cost_scale_factor.powf(simulation.tech_debt)).floor()
    }
}

fn process_new_installed_upgrades(
    mut events: EventReader<UpgradeEvent>,
    mut upgrade_list: ResMut<UpgradeList>,
    mut simulation: ResMut<Simulation>,
) {
    for event in events.read() {
        let upgrade = &mut upgrade_list[event.0];
        upgrade.remaining -= 1;
        simulation.upgrades += 1;
        simulation.tech_debt += upgrade.tech_debt;
        simulation.presentation_score += upgrade.presentation_score;
        simulation.fun_score += upgrade.fun_score;
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
        let Upgrade { install, run, .. } = world.resource::<UpgradeList>()[event.0];
        if let Some(install) = install {
            world.run_system(install).unwrap();
        }
        if let Some(run) = run {
            world.resource_mut::<UpgradeUpdateSystems>().0.push(run);
        }
    }

    // Update all upgrades
    for kind in ALL_UPGRADE_KINDS {
        if let Some(update) = world.resource::<UpgradeList>()[kind].update {
            world.run_system(update).unwrap();
        }
    }
}

#[derive(Resource, Default)]
struct UpgradeUpdateSystems(Vec<SystemId>);

fn run_installed_upgrades(world: &mut World) {
    #[allow(clippy::unnecessary_to_owned)]
    for run in world.resource::<UpgradeUpdateSystems>().0.to_vec() {
        world.run_system(run).unwrap();
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
pub const INITIAL_UPGRADES: [UpgradeKind; 6] = [
    UpgradeKind::DarkMode,
    UpgradeKind::TouchOfLifePlugin,
    UpgradeKind::MovementPlugin,
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

    // Tutorial upgrades

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
    MovementPlugin: Upgrade {
        name: "MovementPlugin".to_string(),
        description: "Allows entities to move. Makes your game more fun.".to_string(),
        fun_score: 5.0,
        base_cost: 5.0,
        install: Some(world.register_system(|
            mut physics_settings: ResMut<PhysicsSettings>,
        | {
            physics_settings.speed_multiplier = UNIT_SPEED;
        })),
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
        base_cost: 2.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: usize::MAX,
        install: Some(
            world.register_system(|mut events: EventWriter<SpawnEvent>, bounds: Res<SceneViewBounds>| {
                events.send(SpawnEvent {
                    position: (bounds.min.xy() + bounds.max.xy()) / 2.0,
                    count: 32.0,
                });
            }),
        ),
        ..default()
    },

    // Upgrades that make your game prettier

    EntitySkinPlugin: Upgrade {
        name: "EntitySkinPlugin".to_string(),
        description: "Adds a new entity skin with a random color. Makes your game prettier.".to_string(),
        presentation_score: 10.0,
        base_cost: 10.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 5,
        install: Some(
            world.register_system(|mut simulation: ResMut<Simulation>| {
                let mut rng = rand::thread_rng();
                simulation.entity_colors.push(
                    Color::Rgba {
                        red: rng.gen_range(0.0..1.0),
                        green: rng.gen_range(0.0..1.0),
                        blue: rng.gen_range(0.0..1.0),
                        alpha: 1.0,
                    }
                );
            }),
        ),
        ..default()
    },
    EntitySizePlugin: Upgrade {
        name: "EntitySizePlugin".to_string(),
        description: "Increases the maximum entity size. Makes your game prettier.".to_string(),
        presentation_score: 10.0,
        base_cost: 10.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 2,
        install: Some(
            world.register_system(|mut simulation: ResMut<Simulation>| {
                simulation.entity_size_max += 4.0;
            }),
        ),
        ..default()
    },

    // Upgrades that make your game more fun

    SpeedupPlugin: Upgrade {
        name: "SpeedupPlugin".to_string(),
        description: "Increases the entity movement speed. Makes your game more fun.".to_string(),
        fun_score: 10.0,
        base_cost: 10.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 5,
        install: Some(world.register_system(|
            mut physics_settings: ResMut<PhysicsSettings>,
        | {
            physics_settings.speed_multiplier += UNIT_SPEED;
        })),
        ..default()
    },

    // Passive entity spawning

    EntitySpawner: Upgrade {
        name: "EntitySpawnerPlugin".to_string(),
        description: "Spawns 1 entity every 2 seconds.".to_string(),
        base_cost: 100.0,
        tech_debt: 1.0,
        weight: 1.0,
        remaining: 1,
        install: Some(world.register_system(|mut entity_spawner: ResMut<PassiveEntitySpawner>| {
            entity_spawner.amount = 1.0;
        })),
        ..default()
    },
    BatchSpawner: Upgrade {
        name: "BatchSpawnerPlugin".to_string(),
        description: "Doubles the amount of entities spawned by EntitySpawnerPlugin.".to_string(),
        requirements: vec![(UpgradeKind::EntitySpawner, 1)],
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        tech_debt: 1.0,
        weight: 0.5,
        remaining: 6,
        install: Some(world.register_system(|mut entity_spawner: ResMut<PassiveEntitySpawner>| {
            entity_spawner.amount *= 2.0;
        })),
        ..default()
    },
    OptimizeSpawner: Upgrade {
        name: "Optimize Spawner".to_string(),
        description: "Halves the cooldown of EntitySpawnerPlugin by optimizing its code.".to_string(),
        requirements: vec![(UpgradeKind::EntitySpawner, 1)],
        base_cost: 100.0,
        cost_scale_factor: 1.2,
        tech_debt: 2.0,
        weight: 0.5,
        remaining: 8,
        install: Some(world.register_system(|mut entity_spawner: ResMut<PassiveEntitySpawner>| {
            let new_duration = entity_spawner.timer.duration().div_f64(2.0);
            entity_spawner.timer.set_duration(new_duration);
        })),
        ..default()
    },

    // Programming upgrades

    ImportLibrary: Upgrade {
        name: "Import Library".to_string(),
        description: "Writes 32 lines of code immediately.".to_string(),
        base_cost: 1.0,
        tech_debt: 1.0,
        weight: 1.0,
        remaining: usize::MAX,
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.lines += 32.0;
        })),
        ..default()
    },
    Refactor: Upgrade {
        name: "Refactor".to_string(),
        description: "Improves the quality of the codebase.".to_string(),
        base_cost: 10.0,
        cost_scale_factor: 1.3,
        tech_debt: -5.0,
        weight: 2.0,
        remaining: usize::MAX,
        tech_debt_min: 15.0,
        ..default()
    },
    TenXDev: Upgrade {
        name: "10x Dev".to_string(),
        description: "Multiplies the number of characters typed per key press by 10.".to_string(),
        base_cost: 100.0,
        tech_debt: 0.0,
        weight: 0.5,
        install: Some(world.register_system(|mut typer_query: Query<&mut CodeTyper>| {
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 10;
            }
        })),
        ..default()
    },
    Rtfm: Upgrade {
        name: "RTFM".to_string(),
        description: "Doubles the number of characters typed per key press.".to_string(),
        base_cost: 50.0,
        tech_debt: -1.0,
        weight: 0.5,
        remaining: 4,
        install: Some(world.register_system(|mut typer_query: Query<&mut CodeTyper>| {
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 2;
            }
        })),
        ..default()
    },

    // Passive code generation

    ProceduralMacro: Upgrade {
        name: "ProceduralMacroPlugin".to_string(),
        description: "Writes 1 line of code every 2 seconds.".to_string(),
        base_cost: 50.0,
        tech_debt: 1.0,
        weight: 1.0,
        remaining: 1,
        install: Some(world.register_system(|mut passive_code_gen: ResMut<PassiveCodeGen>| {
            passive_code_gen.increase = 1.0;
        })),
        ..default()
    },
    NewMacro: Upgrade {
        name: "New Macro".to_string(),
        description: "Doubles the amount of code written by ProceduralMacroPlugin.".to_string(),
        requirements: vec![(UpgradeKind::ProceduralMacro, 1)],
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        tech_debt: 1.0,
        weight: 0.5,
        remaining: 5,
        install: Some(world.register_system(|mut passive_code_gen: ResMut<PassiveCodeGen>| {
            passive_code_gen.increase *= 2.0;
        })),
        ..default()
    },
    DynamicLinking: Upgrade {
        name: "Dynamic Linking".to_string(),
        description: "Halves the cooldown of ProceduralMacroPlugin by speeding up the build process.".to_string(),
        requirements: vec![(UpgradeKind::ProceduralMacro, 1)],
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        tech_debt: 0.0,
        weight: 0.5,
        remaining: 1,
        install: Some(world.register_system(|mut passive_code_gen: ResMut<PassiveCodeGen>| {
            passive_code_gen.timer.set_duration(Duration::from_secs(1));
        })),
        ..default()
    },

    // Miscellaneous

    DesignDocument: Upgrade {
        name: "Design Document".to_string(),
        description: "Adds 1 extra upgrade slot.".to_string(),
        upgrade_min: 7,
        weight: 2.5,
        base_cost: 20.0,
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
);
