use std::ops::Index;
use std::ops::IndexMut;

use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use strum::EnumCount;

use crate::config::Config;
use crate::physics::PhysicsSettings;
use crate::physics::UNIT_SPEED;
use crate::simulation::PassiveCodeTyper;
use crate::simulation::PassiveEntitySpawner;
use crate::simulation::Simulation;
use crate::simulation::SpawnEvent;
use crate::simulation::SpritePack;
use crate::state::editor_screen::spawn_editor_screen;
use crate::state::editor_screen::SceneView;
use crate::state::editor_screen::SceneViewBounds;
use crate::state::editor_screen::UpgradeOutline;
use crate::state::AppState;
use crate::ui::CodeTyper;
use crate::util::pretty_num;
use crate::AppRoot;
use crate::AppSet;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UpgradeEvent>()
            .register_type::<UpgradeSequence>()
            .add_event::<UpgradeEvent>()
            .init_resource::<UpgradeList>()
            .init_resource::<UpgradeUpdateSystems>()
            .add_systems(
                OnEnter(AppState::EditorScreen),
                (load_upgrade_list, load_upgrade_sequence),
            )
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
    /// The name of the upgrade.
    pub name: String,
    /// The desc of the upgrade, with "VALUE" standing in for `self.value`.
    pub desc: String,
    /// How much this upgrade contributes to the Presentation score of your submission.
    pub presentation_score: f64,
    /// How much this upgrade contributes to the Fun score of your submission.
    pub fun_score: f64,
    /// The amount of technical debt this upgrade adds when you install it.
    pub tech_debt: f64,
    /// How many lines of code this upgrade costs without tech debt scaling.
    pub base_cost: f64,
    /// The multiplier to the cost of this upgrade per unit of technical debt.
    pub cost_scale_factor: f64,

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
    /// A list of (upgrade, minimum) that must be installed for this upgrade to be offered.
    pub installed_min: Vec<(UpgradeKind, usize)>,
    /// A list of (upgrade, maximum) allowed to be installed for this upgrade to be offered.
    pub installed_max: Vec<(UpgradeKind, usize)>,
    /// The relative odds of this upgrade being offered.
    pub weight: f32,
    /// How many more copies of this upgrade can be installed.
    pub remaining: usize,

    /// An updatable value to be used by the upgrade for some purpose.
    pub value: f64,
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
            desc: "Undefined.".to_string(),
            presentation_score: 0.0,
            fun_score: 0.0,
            tech_debt: 1.0,
            base_cost: 0.0,
            cost_scale_factor: 1.0,

            entity_min: 0.0,
            entity_max: f64::INFINITY,
            line_min: 0.0,
            line_max: f64::INFINITY,
            upgrade_min: 0,
            upgrade_max: usize::MAX,
            tech_debt_min: f64::NEG_INFINITY,
            tech_debt_max: f64::INFINITY,
            installed_min: vec![],
            installed_max: vec![],
            weight: 0.0,
            remaining: 1,

            value: 0.0,
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
                .installed_min
                .iter()
                .all(|(kind, min)| outline.0.get(kind).unwrap_or(&0) >= min)
            && self
                .installed_max
                .iter()
                .all(|(kind, max)| outline.0.get(kind).unwrap_or(&0) <= max)
    }

    pub fn cost(&self, simulation: &Simulation) -> f64 {
        (self.base_cost * self.cost_scale_factor.powf(simulation.tech_debt)).floor()
    }

    pub fn description(&self) -> String {
        self.desc.replace("VALUE", &pretty_num(self.value))
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

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct UpgradeSequence {
    sequence: Vec<Vec<UpgradeKind>>,
    next_idx: usize,
    slots: usize,
}

impl UpgradeSequence {
    fn new(sequence: Vec<Vec<UpgradeKind>>) -> Self {
        Self {
            sequence,
            next_idx: 0,
            slots: 1,
        }
    }

    pub fn next(
        &mut self,
        upgrade_list: &UpgradeList,
        simulation: &Simulation,
        outline: &UpgradeOutline,
    ) -> Vec<UpgradeKind> {
        // Use the initial sequence of upgrades first
        while self.next_idx < self.sequence.len() {
            self.next_idx += 1;
            let upgrades = self.sequence[self.next_idx - 1]
                .iter()
                .copied()
                .filter(|&kind| upgrade_list[kind].is_unlocked(simulation, outline))
                .collect::<Vec<_>>();

            if !upgrades.is_empty() {
                return upgrades;
            }
        }

        // Filter the list of all upgrade kinds into just the ones that are unlocked
        // Then, (weighted) randomly choose from those upgrades for the available slots
        ALL_UPGRADE_KINDS
            .into_iter()
            .filter(|&kind| {
                let upgrade = &upgrade_list[kind];
                // This prevents the tutorial upgrades from being offered when
                // all other upgrades are exhausted.
                upgrade.weight > 0.0 && upgrade.is_unlocked(simulation, outline)
            })
            .collect::<Vec<_>>()
            .choose_multiple_weighted(&mut thread_rng(), self.slots, |&kind| {
                upgrade_list[kind].weight
            })
            .unwrap()
            .copied()
            .collect::<Vec<_>>()
    }
}

/// Loads the sequence of upgrades offered.
fn load_upgrade_sequence(mut commands: Commands) {
    use UpgradeKind::*;

    commands.insert_resource(UpgradeSequence::new(vec![
        vec![DarkModeDracula, DarkModeBamboo],
        vec![TouchOfLifePlugin],
        vec![Inspiration],
        vec![VelocityPlugin],
        vec![ImportLibrary, SplashOfLifePlugin],
        vec![Coffee, OneBitSpritePack],
        vec![Brainstorm],
    ]));
}

/// A macro that generates UpgradeKind enum and load_upgrade_list system from the given
/// UpgradeKind: Upgrade pairs.
macro_rules! generate_upgrade_list {
    (|$world:ident| $($enumname:ident: $upgrade:expr),+ $(,)?) => {
        /// Enum containing all upgrade types.
        #[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash, EnumCount, Debug)]
        pub enum UpgradeKind {
            $($enumname),+
        }

        pub const ALL_UPGRADE_KINDS: [UpgradeKind; UpgradeKind::COUNT] = [
            $(UpgradeKind::$enumname),+
        ];

        /// A system that initializes and inserts the UpgradeList resource.
        fn load_upgrade_list($world: &mut World) {
            use UpgradeKind::*;

            let upgrade_list = UpgradeList(vec![
                $($upgrade),+
            ]);

            $world.insert_resource(upgrade_list);
        }
    };
}

generate_upgrade_list!(
    |world|

    // Exposition

    Inspiration: Upgrade {
        name: "Inspiration".to_string(),
        desc: "Allows new types of upgrades to unlock when you have enough entities.".to_string(),
        tech_debt: 0.0,
        ..default()
    },

    // Presentation score

    OneBitSpritePack: Upgrade {
        name: "1-bit Sprite Pack".to_string(),
        desc: "Downloads a 1-bit sprite pack for your entities. Makes your game prettier.".to_string(),
        presentation_score: 10.0,
        base_cost: 25.0,
        install: Some(world.register_system(|
            mut simulation: ResMut<Simulation>
        | {
            simulation.sprite_pack = SpritePack::OneBit(vec![]);
            simulation.sprite_pack.add_skin(&mut thread_rng());
        })),
        ..default()
    },
    EntitySkinPlugin: Upgrade {
        name: "EntitySkinPlugin".to_string(),
        desc: "Introduces a new entity skin with a random color. Makes your game prettier.".to_string(),
        presentation_score: 4.0,
        base_cost: 10.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 5,
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.sprite_pack.add_skin(&mut thread_rng());
        })),
        ..default()
    },
    EntitySizePlugin: Upgrade {
        name: "EntitySizePlugin".to_string(),
        desc: "Increases the maximum entity size. Makes your game prettier.".to_string(),
        presentation_score: 2.0,
        base_cost: 10.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 2,
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.entity_size_max += 4.0;
        })),
        ..default()
    },

    // Fun score

    VelocityPlugin: Upgrade {
        name: "VelocityPlugin".to_string(),
        desc: "Allows entities to move. Makes your game more fun.".to_string(),
        fun_score: 5.0,
        base_cost: 5.0,
        install: Some(world.register_system(|
            mut physics_settings: ResMut<PhysicsSettings>,
        | {
            physics_settings.speed_multiplier = UNIT_SPEED;
        })),
        ..default()
    },

    SpeedupPlugin: Upgrade {
        name: "SpeedupPlugin".to_string(),
        desc: "Increases the entity movement speed. Makes your game more fun.".to_string(),
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

    // Entities (immediate)

    SplashOfLifePlugin: Upgrade {
        name: "SplashOfLifePlugin".to_string(),
        desc: "Spawns VALUE entities immediately.".to_string(),
        base_cost: 2.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: usize::MAX,
        update: Some(
            world.register_system(|
                mut upgrade_list: ResMut<UpgradeList>,
                simulation: Res<Simulation>,
            | {
                upgrade_list[SplashOfLifePlugin].value = (simulation.entities * 0.1).max(32.0).floor();
            }),
        ),
        install: Some(
            world.register_system(|
                mut events: EventWriter<SpawnEvent>,
                upgrade_list: Res<UpgradeList>,
                bounds: Res<SceneViewBounds>,
            | {
                events.send(SpawnEvent {
                    position: (bounds.min.xy() + bounds.max.xy()) / 2.0,
                    count: upgrade_list[SplashOfLifePlugin].value,
                });
            }),
        ),
        ..default()
    },

    // Entities (manual)

    TouchOfLifePlugin: Upgrade {
        name: "TouchOfLifePlugin".to_string(),
        desc: "Spawns 1 entity whenever you click inside the scene view.".to_string(),
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
    Coffee: Upgrade {
        name: "Coffee".to_string(),
        desc: "Doubles the number of entities spawned per click.".to_string(),
        base_cost: 25.0,
        weight: 1.0,
        remaining: 3,
        install: Some(
            world.register_system(|mut scene_view_query: Query<&mut SceneView>| {
                for mut scene_view in &mut scene_view_query {
                    scene_view.spawns_per_click *= 2;
                }
            }),
        ),
        ..default()
    },

    // Entities (automatic)

    EntitySpawnerPlugin: Upgrade {
        name: "EntitySpawnerPlugin".to_string(),
        desc: "Spawns 1 entity every 2 seconds.".to_string(),
        base_cost: 100.0,
        weight: 1.0,
        install: Some(world.register_system(|mut entity_spawner: ResMut<PassiveEntitySpawner>| {
            entity_spawner.amount += 1.0;
        })),
        ..default()
    },
    BatchSpawnerPlugin: Upgrade {
        name: "BatchSpawnerPlugin".to_string(),
        desc: "Doubles the number of entities spawned by EntitySpawnerPlugin.".to_string(),
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        installed_min: vec![(EntitySpawnerPlugin, 1)],
        weight: 0.5,
        remaining: 6,
        install: Some(world.register_system(|mut entity_spawner: ResMut<PassiveEntitySpawner>| {
            entity_spawner.amount *= 2.0;
        })),
        ..default()
    },
    OptimizeSpawner: Upgrade {
        name: "Optimize Spawner".to_string(),
        desc: "Halves the cooldown of EntitySpawnerPlugin with some clever optimizations.".to_string(),
        tech_debt: 2.0,
        base_cost: 100.0,
        cost_scale_factor: 1.2,
        installed_min: vec![(EntitySpawnerPlugin, 1)],
        weight: 0.5,
        remaining: 8,
        install: Some(world.register_system(|mut entity_spawner: ResMut<PassiveEntitySpawner>| {
            let new_duration = entity_spawner.timer.duration().div_f64(2.0);
            entity_spawner.timer.set_duration(new_duration);
        })),
        ..default()
    },

    // Lines (immediate)

    ImportLibrary: Upgrade {
        name: "Import Library".to_string(),
        desc: "Writes VALUE lines of code immediately.".to_string(),
        base_cost: 1.0,
        weight: 1.0,
        remaining: usize::MAX,
        update: Some(
            world.register_system(|
                mut upgrade_list: ResMut<UpgradeList>,
                simulation: Res<Simulation>,
            | {
                upgrade_list[ImportLibrary].value = (simulation.lines * 0.1).max(32.0).floor();
            }),
        ),
        install: Some(world.register_system(|
            upgrade_list: Res<UpgradeList>,
            mut simulation: ResMut<Simulation>,
        | {
            simulation.lines += upgrade_list[ImportLibrary].value;
        })),
        ..default()
    },

    // Themes

    DarkModeDracula: Upgrade {
        name: "Dark Mode (Dracula)".to_string(),
        desc: "Rite of passage for all developers. Required to write code.".to_string(),
        tech_debt: 0.0,
        install: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(
                &mut commands,
                config.editor_screen.dracula_theme.clone(),
                false,
            );
            commands.entity(editor_screen).set_parent(root.ui);
        })),
        ..default()
    },
    DarkModeBamboo: Upgrade {
        name: "Dark Mode (Bamboo)".to_string(),
        desc: "Rite of passage for all developers. Required to write code.".to_string(),
        tech_debt: 0.0,
        install: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(
                &mut commands,
                config.editor_screen.bamboo_theme.clone(),
                false,
            );
            commands.entity(editor_screen).set_parent(root.ui);
        })),
        ..default()
    },

    // Lines (manual)

    MechanicalKeyboard: Upgrade {
        name: "Mechanical Keyboard".to_string(),
        desc: "Doubles the number of characters typed per key press.".to_string(),
        tech_debt: 0.0,
        base_cost: 50.0,
        weight: 0.5,
        install: Some(world.register_system(|mut typer_query: Query<&mut CodeTyper>| {
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 2;
            }
        })),
        ..default()
    },
    TenXDev: Upgrade {
        name: "10x Dev".to_string(),
        desc: "Multiplies the number of characters typed per key press by 10.".to_string(),
        tech_debt: 0.0,
        base_cost: 100.0,
        weight: 0.5,
        install: Some(world.register_system(|mut typer_query: Query<&mut CodeTyper>| {
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 10;
            }
        })),
        ..default()
    },

    // Lines (automatic)

    ProceduralMacro: Upgrade {
        name: "Procedural Macro".to_string(),
        desc: "Writes 30 characters every 2 seconds.".to_string(),
        base_cost: 50.0,
        weight: 1.0,
        remaining: 1,
        install: Some(world.register_system(|mut typer: ResMut<PassiveCodeTyper>| {
            typer.chars += 30.0;
        })),
        ..default()
    },
    MetaMacro: Upgrade {
        name: "Meta Macro".to_string(),
        desc: "Doubles the output of Procedural Macro.".to_string(),
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        installed_min: vec![(ProceduralMacro, 1)],
        weight: 0.5,
        remaining: 6,
        install: Some(world.register_system(|mut typer: ResMut<PassiveCodeTyper>| {
            typer.chars *= 2.0;
        })),
        ..default()
    },
    OptimizeBuild: Upgrade {
        name: "Optimize Build".to_string(),
        desc: "Halves the cooldown of Procedural Macro by optimizing the build process.".to_string(),
        tech_debt: 0.0,
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        installed_min: vec![(ProceduralMacro, 1)],
        weight: 0.5,
        remaining: 8,
        install: Some(world.register_system(|mut typer: ResMut<PassiveCodeTyper>| {
            let new_duration = typer.timer.duration().div_f64(2.0);
            typer.timer.set_duration(new_duration);
        })),
        ..default()
    },
    LlmPlugin: Upgrade {
        name: "LlmPlugin".to_string(),
        desc: "Inserts an LlmComponent on all existing and future entities. Each LlmComponent writes 1 character every 2 seconds.".to_string(),
        tech_debt: 2.0,
        base_cost: 200.0,
        cost_scale_factor: 1.2,
        entity_min: 1000.0,
        weight: 0.1,
        install: Some(world.register_system(|mut typer: ResMut<PassiveCodeTyper>| {
            typer.chars_per_entity += 1.0;
        })),
        ..default()
    },

    // Technical debt (immediate)

    Refactor: Upgrade {
        name: "Refactor".to_string(),
        desc: "Improves the quality of the codebase.".to_string(),
        tech_debt: -5.0,
        base_cost: 10.0,
        cost_scale_factor: 1.3,
        tech_debt_min: 15.0,
        weight: 2.0,
        remaining: usize::MAX,
        ..default()
    },
    Rtfm: Upgrade {
        name: "RTFM".to_string(),
        desc: "Reduces all future technical debt increases by 10%.".to_string(),
        tech_debt: 0.0,
        base_cost: 20.0,
        tech_debt_min: 5.0,
        weight: 1.0,
        remaining: 4,
        install: Some(world.register_system(|mut upgrade_list: ResMut<UpgradeList>| {
            for upgrade in &mut upgrade_list.0 {
                if upgrade.tech_debt > 0.0 {
                    upgrade.tech_debt *= 0.9;
                }
            }
        })),
        ..default()
    },

    // Slots (immediate)

    Brainstorm: Upgrade {
        name: "Brainstorm".to_string(),
        desc: "Adds 1 extra upgrade slot.".to_string(),
        tech_debt: 0.0,
        install: Some(world.register_system(|mut sequence: ResMut<UpgradeSequence>| {
            sequence.slots += 1;
        })),
        ..default()
    },
    DesignDocument: Upgrade {
        name: "Design Document".to_string(),
        desc: "Adds 1 extra upgrade slot.".to_string(),
        tech_debt: 0.0,
        base_cost: 20.0,
        upgrade_min: 7,
        weight: 2.5,
        install: Some(world.register_system(|mut sequence: ResMut<UpgradeSequence>| {
            sequence.slots += 1;
        })),
        ..default()
    },
);
