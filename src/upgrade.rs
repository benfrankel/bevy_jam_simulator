use std::ops::Index;
use std::ops::IndexMut;

use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use strum::EnumCount;

use crate::audio::AudioAssets;
use crate::audio::BackgroundMusic;
use crate::audio::SoundEffectKind;
use crate::config::Config;
use crate::physics::PhysicsSettings;
use crate::physics::Velocity;
use crate::physics::UNIT_SPEED;
use crate::simulation::AtlasList;
use crate::simulation::LinesAddedEvent;
use crate::simulation::PassiveCodeTyper;
use crate::simulation::PassiveEntitySpawner;
use crate::simulation::Simulation;
use crate::simulation::SpawnEvent;
use crate::simulation::SpritePack;
use crate::simulation::SpritePackEvent;
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
    /// An updatable value to be used by the upgrade for some purpose.
    pub value: f64,
    /// The sound effect this upgrade will emit upon install.
    pub sound: Option<SoundEffectKind>,
    /// If true, this upgrade won't be added to the outline and won't count as an upgrade.
    pub no_outline: bool,
    /// If true, this upgrade's count will not be included in the outline.
    pub no_count: bool,
    /// The amount of technical debt this upgrade adds when you install it.
    pub tech_debt: f64,
    /// How much this upgrade contributes to the Presentation score of your submission.
    pub presentation_score: f64,
    /// How much this upgrade contributes to the Fun score of your submission.
    pub fun_score: f64,
    /// How many lines of code this upgrade costs without tech debt scaling.
    pub base_cost: f64,
    /// The multiplier to the cost of this upgrade per unit of technical debt.
    pub cost_scale_factor: f64,

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
    /// A list of (upgrade, minimum) that must be installed for this upgrade to be offered.
    pub installed_min: Vec<(UpgradeKind, usize)>,
    /// A list of (upgrade, maximum) allowed to be installed for this upgrade to be offered.
    pub installed_max: Vec<(UpgradeKind, usize)>,

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
            value: 0.0,
            sound: Some(SoundEffectKind::DefaultUpgrade),
            no_outline: false,
            no_count: false,
            tech_debt: 0.0,
            presentation_score: 0.0,
            fun_score: 0.0,
            base_cost: 0.0,
            cost_scale_factor: 1.0,

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
            installed_min: vec![],
            installed_max: vec![],

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
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    for event in events.read() {
        let upgrade = &mut upgrade_list[event.kind];
        upgrade.remaining -= 1;
        simulation.tech_debt += upgrade.tech_debt;
        simulation.presentation_score += upgrade.presentation_score;
        simulation.fun_score += upgrade.fun_score;
        if !upgrade.no_outline {
            simulation.upgrades += 1;
        }
        if let Some(sound) = upgrade.sound {
            audio.play(audio_assets.get_sfx(sound));
        }
    }
}

#[derive(Event, Reflect, Clone)]
pub struct UpgradeEvent {
    pub kind: UpgradeKind,
    // This is a hack to deal with names/descs that change on install.
    pub name: String,
    pub desc: String,
}

fn install_upgrades(world: &mut World, mut reader: Local<ManualEventReader<UpgradeEvent>>) {
    for event in reader
        .read(world.resource::<Events<_>>())
        .cloned()
        .collect::<Vec<_>>()
    {
        let Upgrade { install, run, .. } = world.resource::<UpgradeList>()[event.kind];
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
    /// A stack of upcoming upgrades. If this is non-empty, the next set of upgrades will
    /// be the given list and the refresh button won't be offered. The given string will
    /// be rendered beneath the list.
    stack: Vec<(Vec<UpgradeKind>, String)>,
    slots: usize,
}

impl UpgradeSequence {
    fn new(mut sequence: Vec<(Vec<UpgradeKind>, String)>) -> Self {
        sequence.reverse();
        Self {
            stack: sequence,
            slots: 1,
        }
    }

    pub fn push(&mut self, options: Vec<UpgradeKind>, desc: String) {
        self.stack.push((options, desc));
    }

    /// Get the next upgrade list, with optional description.
    pub fn next(
        &mut self,
        upgrade_list: &UpgradeList,
        simulation: &Simulation,
        outline: &UpgradeOutline,
    ) -> (Vec<UpgradeKind>, String) {
        // Check the stack of upgrades first
        while let Some((upgrades, desc)) = self.stack.pop() {
            let upgrades = upgrades
                .iter()
                .copied()
                .filter(|&kind| upgrade_list[kind].is_unlocked(simulation, outline))
                .collect::<Vec<_>>();

            if !upgrades.is_empty() {
                return (upgrades, desc);
            }
        }

        // Filter the list of all upgrade kinds into just the ones that are unlocked
        // Then, (weighted) randomly choose from those upgrades for the available slots
        let mut upgrades = ALL_UPGRADE_KINDS
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
            .collect::<Vec<_>>();

        // Sort by UpgradeKind order
        upgrades.sort();

        // Add an upgrade that refreshes the upgrade list to reduce the dependency on luck.
        upgrades.push(UpgradeKind::RefreshUpgradeList);

        (upgrades, String::new())
    }
}

/// Loads the sequence of upgrades offered.
fn load_upgrade_sequence(mut commands: Commands) {
    use UpgradeKind::*;

    commands.insert_resource(UpgradeSequence::new(vec![
        (
            vec![DarkModeDracula, DarkModeBamboo, DarkModeSynthwave],
            String::new(),
        ),
        (
            vec![Inspiration],
            "\"Much better. Now I can get started.\"".to_string(),
        ),
        (
            vec![TouchOfLifePlugin],
            "\"I don't know what I'm making, but I should start spawning entities.\"".to_string(),
        ),
        (
            vec![VelocityPlugin],
            "\"I should make the game more interesting for a higher Fun score.\"".to_string(),
        ),
        (vec![ImportLibrary, SplashOfLifePlugin], String::new()),
        (
            vec![SkinPlugin, Coffee],
            "\"I should also make the game look pretty for a higher Presentation score.\""
                .to_string(),
        ),
        (
            vec![Brainstorm],
            "\"Hmm... where should I go from here?\"".to_string(),
        ),
    ]));
}

/// A macro that generates UpgradeKind enum and load_upgrade_list system from the given
/// UpgradeKind: Upgrade pairs.
macro_rules! generate_upgrade_list {
    (|$world:ident| $($enumname:ident: $upgrade:expr),+ $(,)?) => {
        /// Enum containing all upgrade types.
        #[derive(Reflect, Clone, Copy, PartialEq, Eq, Hash, EnumCount, Debug, PartialOrd, Ord)]
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
        no_outline: true,
        base_cost: 5.0,
        ..default()
    },

    // Presentation score

    GfxSpecialization: Upgrade {
        name: "GFX Specialization".to_string(),
        desc: "Offers a choice between different graphics options that affect how your game looks.".to_string(),
        no_outline: true,
        base_cost: 1.0,
        weight: 2.0,
        entity_min: 35.0,
        install: Some(world.register_system(|mut sequence: ResMut<UpgradeSequence>| {
            sequence.push(
                vec![SpritePackOneBit, SpritePackRpg, SpritePackNinja, OptimizeShaders],
                "You can only select one option. \
                 The rejected options will never appear again.".to_string(),
            );
        })),
        ..default()
    },

    SpritePackOneBit: Upgrade {
        name: "Sprite Pack (1-bit)".to_string(),
        desc: "Downloads a 1-bit sprite pack for your entities. Makes your game prettier.".to_string(),
        tech_debt: 1.0,
        presentation_score: 5.0,
        base_cost: 20.0,
        install: Some(world.register_system(|
            mut events: EventWriter<SpritePackEvent>,
            mut simulation: ResMut<Simulation>,
            atlas_list: Res<AtlasList>,
        | {
            simulation.skin_set.replace_sprite_pack(&atlas_list, SpritePack::OneBit, &mut thread_rng());
            events.send(SpritePackEvent);
        })),
        ..default()
    },

    SpritePackRpg: Upgrade {
        name: "Sprite Pack (RPG)".to_string(),
        desc: "Downloads an RPG sprite pack for your entities. Makes your game prettier.".to_string(),
        tech_debt: 1.0,
        presentation_score: 10.0,
        base_cost: 30.0,
        install: Some(world.register_system(|
            mut events: EventWriter<SpritePackEvent>,
            mut simulation: ResMut<Simulation>,
            atlas_list: Res<AtlasList>,
        | {
            simulation.skin_set.replace_sprite_pack(&atlas_list, SpritePack::Rpg, &mut thread_rng());
            events.send(SpritePackEvent);
        })),
        ..default()
    },

    OptimizeShaders: Upgrade {
        name: "Optimize Shaders".to_string(),
        desc: "Instead of installing a sprite pack, optimizes your shaders to render squares faster. Doubles the entity spawn rate but does not make your game look pretty.".to_string(),
        tech_debt: 1.0,
        base_cost: 100.0,
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.entity_spawn_multiplier *= 2.0;
        })),
        ..default()
    },

    SpritePackNinja: Upgrade {
        name: "Sprite Pack (Ninja)".to_string(),
        desc: "Downloads a Ninja sprite pack for your entities. Makes your game prettier.".to_string(),
        tech_debt: 1.0,
        presentation_score: 15.0,
        base_cost: 40.0,
        install: Some(world.register_system(|
            mut events: EventWriter<SpritePackEvent>,
            mut simulation: ResMut<Simulation>,
            atlas_list: Res<AtlasList>,
        | {
            simulation.skin_set.replace_sprite_pack(&atlas_list, SpritePack::Ninja, &mut thread_rng());
            events.send(SpritePackEvent);
        })),
        ..default()
    },

    SkinPlugin: Upgrade {
        name: "SkinPlugin".to_string(),
        desc: "Introduces a new entity skin. Makes your game prettier.".to_string(),
        tech_debt: 1.0,
        presentation_score: 4.0,
        base_cost: 10.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 5,
        install: Some(world.register_system(|
            mut simulation: ResMut<Simulation>,
            atlas_list: Res<AtlasList>,
        | {
            simulation.skin_set.add_skin(&atlas_list, &mut thread_rng());
        })),
        ..default()
    },

    ScalePlugin: Upgrade {
        name: "ScalePlugin".to_string(),
        desc: "Increases the maximum entity size. Makes your game prettier.".to_string(),
        tech_debt: 1.0,
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
        tech_debt: 1.0,
        fun_score: 5.0,
        base_cost: 5.0,
        install: Some(world.register_system(|
            mut physics_settings: ResMut<PhysicsSettings>,
        | {
            physics_settings.speed_multiplier = UNIT_SPEED;
        })),
        ..default()
    },

    SpeedPlugin: Upgrade {
        name: "SpeedPlugin".to_string(),
        desc: "Increases the entity movement speed. Makes your game more fun.".to_string(),
        tech_debt: 1.0,
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

    NuclearBlastPlugin: Upgrade {
        name: "NuclearBlastPlugin".to_string(),
        desc: "Destroys all entities but makes your game a lot more fun.".to_string(),
        tech_debt: 1.0,
        base_cost: 1_000_000.0,
        fun_score: 100.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        entity_min: 1_000_000.0,
        remaining: 3,
        install: Some(
            world.register_system(|
                mut query: Query<&mut Visibility, With<Velocity>>,
                mut simulation: ResMut<Simulation>,
                mut upgrade_list: ResMut<UpgradeList>,
            | {
                for mut visibility in &mut query {
                    *visibility = Visibility::Hidden;
                }
                simulation.entities = 0.0;
                let this = &mut upgrade_list[NuclearBlastPlugin];
                this.entity_min *= 1_000_000.0;
            }),
        ),
        ..default()
    },

    // Entities (immediate)

    SplashOfLifePlugin: Upgrade {
        name: "SplashOfLifePlugin".to_string(),
        desc: "Spawns VALUE entities immediately.".to_string(),
        tech_debt: 1.0,
        base_cost: 2.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: usize::MAX,
        update: Some(
            world.register_system(|
                mut upgrade_list: ResMut<UpgradeList>,
                simulation: Res<Simulation>,
            | {
                let this = &mut upgrade_list[SplashOfLifePlugin];
                this.value = (simulation.entities * 0.1).max(32.0).floor();
            }),
        ),
        install: Some(
            world.register_system(|
                mut events: EventWriter<SpawnEvent>,
                upgrade_list: Res<UpgradeList>,
                bounds: Res<SceneViewBounds>,
            | {
                let this = &upgrade_list[SplashOfLifePlugin];
                events.send(SpawnEvent {
                    position: (bounds.min.xy() + bounds.max.xy()) / 2.0,
                    count: this.value,
                });
            }),
        ),
        ..default()
    },

    // Entities (manual)

    TouchOfLifePlugin: Upgrade {
        name: "TouchOfLifePlugin".to_string(),
        desc: "Spawns 1 entity whenever you click inside the scene view.".to_string(),
        tech_debt: 1.0,
        base_cost: 5.0,
        install: Some(
            world.register_system(|mut scene_view_query: Query<&mut SceneView>| {
                for mut scene_view in &mut scene_view_query {
                    scene_view.spawns_per_click += 1.0;
                }
            }),
        ),
        ..default()
    },

    Coffee: Upgrade {
        name: "Coffee".to_string(),
        desc: "Quadruples the number of entities spawned per click.".to_string(),
        base_cost: 25.0,
        weight: 1.0,
        remaining: 3,
        install: Some(
            world.register_system(|mut scene_view_query: Query<&mut SceneView>| {
                for mut scene_view in &mut scene_view_query {
                    scene_view.spawns_per_click *= 4.0;
                }
            }),
        ),
        ..default()
    },

    // Entities (automatic)

    EntitySpawnerPlugin: Upgrade {
        name: "EntitySpawnerPlugin".to_string(),
        desc: "Spawns 1 entity every 2 seconds.".to_string(),
        tech_debt: 1.0,
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
        tech_debt: 1.0,
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        weight: 0.5,
        remaining: 6,
        installed_min: vec![(EntitySpawnerPlugin, 1)],
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
        weight: 0.5,
        remaining: 8,
        installed_min: vec![(EntitySpawnerPlugin, 1)],
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
        tech_debt: 1.0,
        base_cost: 1.0,
        weight: 1.0,
        remaining: usize::MAX,
        update: Some(
            world.register_system(|
                mut upgrade_list: ResMut<UpgradeList>,
                simulation: Res<Simulation>,
            | {
                let this = &mut upgrade_list[ImportLibrary];
                this.value = (simulation.total_lines * 0.1).max(32.0).floor();
            }),
        ),
        install: Some(world.register_system(|
            mut events: EventWriter<LinesAddedEvent>,
            upgrade_list: Res<UpgradeList>,
        | {
            let this = &upgrade_list[ImportLibrary];
            events.send(LinesAddedEvent { count: this.value });
        })),
        ..default()
    },

    // TODO: These would be better implemented by sending e.g. a ChangeEditorTheme event
    // Editor themes

    DarkModeDracula: Upgrade {
        name: "Dark Mode (Dracula)".to_string(),
        desc: "Rite of passage for all developers. Required to write code.".to_string(),
        sound: None,
        install: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
            music: Res<BackgroundMusic>,
            mut audio_instances: ResMut<Assets<AudioInstance>>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(
                &mut commands,
                &config.editor_screen,
                &config.editor_screen.dracula_theme,
                false,
            );
            commands.entity(editor_screen).set_parent(root.ui);

            // Start background music
            if let Some(instance) = audio_instances.get_mut(&music.0) {
                instance.resume(AudioTween::default());
            } else {
                error!("Background music has not loaded yet");
            }
        })),
        ..default()
    },

    DarkModeBamboo: Upgrade {
        name: "Dark Mode (Bamboo)".to_string(),
        desc: "Rite of passage for all developers. Required to write code.".to_string(),
        sound: None,
        install: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
            music: Res<BackgroundMusic>,
            mut audio_instances: ResMut<Assets<AudioInstance>>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(
                &mut commands,
                &config.editor_screen,
                &config.editor_screen.bamboo_theme,
                false,
            );
            commands.entity(editor_screen).set_parent(root.ui);

            // Start background music
            if let Some(instance) = audio_instances.get_mut(&music.0) {
                instance.resume(AudioTween::default());
            } else {
                error!("Background music has not loaded yet");
            }
        })),
        ..default()
    },

    DarkModeSynthwave: Upgrade {
        name: "Dark Mode (Synthwave)".to_string(),
        desc: "Rite of passage for all developers. Required to write code.".to_string(),
        sound: None,
        install: Some(world.register_system(|
            mut commands: Commands,
            root: Res<AppRoot>,
            config: Res<Config>,
            music: Res<BackgroundMusic>,
            mut audio_instances: ResMut<Assets<AudioInstance>>,
        | {
            commands.entity(root.ui).despawn_descendants();
            let editor_screen = spawn_editor_screen(
                &mut commands,
                &config.editor_screen,
                &config.editor_screen.synthwave_theme,
                false,
            );
            commands.entity(editor_screen).set_parent(root.ui);

            // Start background music
            if let Some(instance) = audio_instances.get_mut(&music.0) {
                instance.resume(AudioTween::default());
            } else {
                error!("Background music has not loaded yet");
            }
        })),
        ..default()
    },

    // Lines (manual)

    MechanicalKeyboard: Upgrade {
        name: "Mechanical Keyboard".to_string(),
        desc: "\
            A better keyboard that allows you to type faster. \
            Doubles the number of characters typed per key press. \
        ".to_string(),
        sound: Some(SoundEffectKind::Keyboard),
        no_count: true,
        base_cost: 50.0,
        weight: 2.0,
        remaining: 2,
        install: Some(world.register_system(|
            mut typer_query: Query<&mut CodeTyper>,
            mut upgrade_list: ResMut<UpgradeList>,
        | {
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 2;
            }
            // Update this upgrade for the next iteration: Ergonomic Keyboard.
            let this = &mut upgrade_list[MechanicalKeyboard];
            // Cost scaling of this is independent of tech debt.
            this.base_cost *= 4.0;
            this.weight = 0.5;
            this.installed_min.push((TouchTyping, 1));
            this.name = "Ergonomic Keyboard".to_string();
            this.desc = "\
                An even better keyboard that allows you to type faster. \
                Quadruples the number of characters typed per key press. \
                Replaces the mechanical keyboard. \
            ".to_string();
        })),
        ..default()
    },
    TouchTyping: Upgrade {
        name: "Touch Typing".to_string(),
        desc: "\
            Improves your typing skills. \
            Doubles the number of characters typed per key press. \
        ".to_string(),
        sound: Some(SoundEffectKind::Keyboard),
        installed_min: vec![(MechanicalKeyboard, 1)],
        base_cost: 100.0,
        weight: 1.0,
        remaining: 4,
        install: Some(world.register_system(|
            mut typer_query: Query<&mut CodeTyper>,
            mut upgrade_list: ResMut<UpgradeList>,
        | {
            let this = &mut upgrade_list[TouchTyping];
            // Cost scaling of this is independent of tech debt.
            this.base_cost *= 2.0;
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 2;
            }
        })),
        ..default()
    },
    DvorakLayout: Upgrade {
        name: "Dvorak Layout".to_string(),
        desc: "Doubles the number of characters typed per key press.".to_string(),
        sound: Some(SoundEffectKind::Keyboard),
        installed_min: vec![(TouchTyping, 4)],
        base_cost: 200_000.0,
        weight: 0.25,
        install: Some(world.register_system(|mut typer_query: Query<&mut CodeTyper>| {
            for mut typer in &mut typer_query {
                typer.chars_per_key *= 2;
            }
        })),
        ..default()
    },

    ProceduralMacro: Upgrade {
        name: "Procedural Macro".to_string(),
        desc: "\
            Writes one line of code for each line you type. \
            Simplifies the codebase slightly. \
        ".to_string(),
        tech_debt: -1.0,
        base_cost: 100.0,
        cost_scale_factor: 1.1,
        weight: 1.0,
        remaining: 1,
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.line_multiplier *= 2.0;
        })),
        ..default()
    },

    MetaMacro: Upgrade {
        name: "Meta Macro".to_string(),
        desc: "Doubles the output of Procedural Macro.".to_string(),
        tech_debt: 1.0,
        base_cost: 200.0,
        cost_scale_factor: 1.2,
        weight: 0.5,
        remaining: 5,
        installed_min: vec![(ProceduralMacro, 1)],
        install: Some(world.register_system(|mut simulation: ResMut<Simulation>| {
            simulation.line_multiplier *= 2.0;
        })),
        ..default()
    },

    // Lines (automatic)

    CodingLlm: {
        // format! expects a string literal. CONST &str doesn't work.
        macro_rules! desc_template {
            () => {"\
                A {} billion parameter large language model that writes code. \
                Types {} characters every 2 seconds. \
            "}
        }
        const PARAMETERS: [u32; 4] = [7, 13, 33, 65];
        const CHARS: [f64; 4] = [30.0, 100.0, 300.0, 1000.0];
        const COSTS: [f64; 4] = [1000.0, 5000.0, 25_000.0, 500_000.0];

        Upgrade {
            name: format!("Coding LLM {}B", PARAMETERS[0]).to_string(),
            desc: format!(desc_template!(), PARAMETERS[0], CHARS[0]).to_string(),
            tech_debt: 1.0,
            base_cost: 1000.0,
            weight: 0.75,
            remaining: 4,
            no_count: true,
            installed_min: vec![(MetaMacro, 1)],
            install: Some(world.register_system(|
                mut typer: ResMut<PassiveCodeTyper>,
                mut upgrade_list: ResMut<UpgradeList>,
            | {
                let this = &mut upgrade_list[CodingLlm];

                let current_idx = 3 - this.remaining;
                typer.chars = CHARS[current_idx];

                let next_idx = current_idx + 1;
                if next_idx < 4 {
                    this.base_cost = COSTS[next_idx];
                    this.name = format!("Coding LLM {}B", PARAMETERS[next_idx]).to_string();
                    this.desc = format!(
                        desc_template!(), PARAMETERS[next_idx], CHARS[next_idx],
                    ).to_string();
                }
            })),
            ..default()
        }
    },

    OptimizeLlm: Upgrade {
        name: "Optimize LLM".to_string(),
        desc: "Halves the cooldown of Coding LLM by optimizing inference.".to_string(),
        base_cost: 1000.0,
        cost_scale_factor: 1.2,
        weight: 0.75,
        remaining: 8,
        installed_min: vec![(CodingLlm, 1)],
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
        weight: 0.25,
        entity_min: 10_000.0,
        installed_min: vec![(CodingLlm, 1), (OptimizeLlm, 2)],
        install: Some(world.register_system(|mut typer: ResMut<PassiveCodeTyper>| {
            typer.chars_per_entity += 1.0;
        })),
        ..default()
    },

    // Technical debt (immediate)

    Refactor: Upgrade {
        name: "Refactor".to_string(),
        desc: "Improves the quality of the codebase.".to_string(),
        sound: Some(SoundEffectKind::Backspace),
        tech_debt: -5.0,
        base_cost: 10.0,
        cost_scale_factor: 1.5,
        weight: 2.0,
        remaining: usize::MAX,
        tech_debt_min: 10.0,
        install: Some(world.register_system(|mut upgrade_list: ResMut<UpgradeList>| {
            let this = &mut upgrade_list[Refactor];
            this.tech_debt_min += 5.0;
        })),
        ..default()
    },

    // Unit test reduces both the technical debt and its multiplier slightly.
    UnitTests: Upgrade {
        name: "Unit Tests".to_string(),
        desc: "Improves the quality of the codebase. Reduces all future technical debt increases by 5%.".to_string(),
        tech_debt: -3.0,
        base_cost: 20.0,
        cost_scale_factor: 1.3,
        weight: 1.0,
        remaining: 2,
        tech_debt_min: 3.0,
        install: Some(world.register_system(|mut upgrade_list: ResMut<UpgradeList>| {
            for upgrade in &mut upgrade_list.0 {
                if upgrade.tech_debt > 0.0 {
                    upgrade.tech_debt *= 0.95;
                }
            }
        })),
        ..default()
    },

    // Technical debt (multiplier)

    Rtfm: Upgrade {
        name: "RTFM".to_string(),
        desc: "Reduces all future technical debt increases by 5%.".to_string(),
        base_cost: 20.0,
        weight: 1.0,
        remaining: 2,
        tech_debt_min: 5.0,
        install: Some(world.register_system(|mut upgrade_list: ResMut<UpgradeList>| {
            for upgrade in &mut upgrade_list.0 {
                if upgrade.tech_debt > 0.0 {
                    upgrade.tech_debt *= 0.95;
                }
            }
        })),
        ..default()
    },

    CiCd: Upgrade {
        name: "CI/CD".to_string(),
        desc: "Reduces all future technical debt increases by 10%.".to_string(),
        tech_debt: 0.5,
        base_cost: 50.0,
        cost_scale_factor: 1.2,
        weight: 1.0,
        remaining: 2,
        installed_min: vec![(Rtfm, 2), (UnitTests, 1)],
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
        install: Some(world.register_system(|mut sequence: ResMut<UpgradeSequence>| {
            sequence.slots += 1;
        })),
        ..default()
    },

    DesignDocument: Upgrade {
        name: "Design Document".to_string(),
        desc: "Adds 1 extra upgrade slot.".to_string(),
        base_cost: 20.0,
        weight: 2.5,
        upgrade_min: 7,
        install: Some(world.register_system(|mut sequence: ResMut<UpgradeSequence>| {
            sequence.slots += 1;
        })),
        ..default()
    },

    // Specialization

    Specialization: Upgrade {
        name: "Specialization".to_string(),
        desc: "Offers a choice between powerful specialization paths.".to_string(),
        no_outline: true,
        base_cost: 100.0,
        weight: 2.5,
        upgrade_min: 20,
        install: Some(world.register_system(|mut sequence: ResMut<UpgradeSequence>| {
            sequence.push(
                vec![TenXDev, RockstarDev],
                "This is a specialization upgrade. \
                 You can only select one path. \
                 The rejected options will never appear again.".to_string(),
            );
        })),
        ..default()
    },

    TenXDev: {
        const NAMES: [&str; 6] = [
            "10x Dev",
            "100x Dev",
            "1,000x Dev",
            "10,000x Dev",
            "100,000x Dev",
            "1,000,000x Dev",
        ];
        let mut name_idx = 0;

        Upgrade {
            name: "10x Dev".to_string(),
            desc: "Multiplies all code generation by VALUE.".to_string(),
            value: 10.0,
            no_count: true,
            remaining: 6,
            install: Some(world.register_system(move |
                mut simulation: ResMut<Simulation>,
                mut upgrade_list: ResMut<UpgradeList>,
            | {
                let this = &mut upgrade_list[TenXDev];

                if name_idx + 1 < NAMES.len() {
                    name_idx += 1;
                    this.name = NAMES[name_idx].to_string();
                }
                this.value *= 10.0;

                if this.remaining == 5 {
                    // First time (remaining is decreased beforehand)
                    simulation.line_multiplier *= 10.0;
                    // Make subsequent copies of this upgrade available in the random pool.
                    this.base_cost = 100.0;
                    this.weight = 1.0;
                } else {
                    // Level-up
                    simulation.line_multiplier *= 10.0;
                }
                // Special scaling
                this.base_cost *= 100.0;
            })),
            ..default()
        }
    },

    RockstarDev: {
        const NAMES: [&str; 6] = [
            "Rockstar Dev",
            "Superstar Dev",
            "Hypergiant Dev",
            "Neutron Star Dev",
            "Black Hole Dev",
            "Quasar Dev",
        ];
        let mut name_idx = 0;

        Upgrade {
            name: NAMES[0].to_string(),
            desc: "Spawns VALUE entities whenever a line of code is produced.".to_string(),
            value: 4.0,
            no_count: true,
            remaining: 6,
            install: Some(world.register_system(move |
                mut simulation: ResMut<Simulation>,
                mut upgrade_list: ResMut<UpgradeList>,
            | {
                let this = &mut upgrade_list[RockstarDev];

                if name_idx + 1 < NAMES.len() {
                    name_idx += 1;
                    this.name = NAMES[name_idx].to_string();
                }
                this.value *= 2.0;

                if this.remaining == 5 {
                    // First time (remaining is decreased beforehand)
                    simulation.entity_spawn_per_line += 4.0;
                    // Make subsequent copies of this upgrade available in the random pool.
                    this.base_cost = 100.0;
                    this.weight = 1.0;
                } else {
                    // Level up
                    simulation.entity_spawn_per_line *= 2.0;
                }
                // Special scaling
                this.base_cost *= 100.0;
            })),
            ..default()
        }
    },

    // Misc

    RefreshUpgradeList: {
        let mut names: [&str; 10] = [
            "Playtest",
            "Think Twice",
            "Drink Water",
            "Take a Nap",
            "Talk to a Friend",
            "Order a Pizza",
            "Go for a Walk",
            "Take a Shower",
            "Read a Book",
            "Watch Clouds",
        ];
        names.shuffle(&mut thread_rng());
        let mut name_idx = 0usize;

        Upgrade {
            name: names[name_idx].to_string(),
            desc: "Refreshes your upgrade options. Costs twice as much next time.".to_string(),
            no_outline: true,
            base_cost: 1.0,
            remaining: usize::MAX,
            install: Some(world.register_system(move |mut list: ResMut<UpgradeList>| {
                let this = &mut list[RefreshUpgradeList];
                // Increase base cost
                this.base_cost *= 2.0;
                // Update name
                name_idx += 1;
                if name_idx >= names.len() {
                    names.shuffle(&mut thread_rng());
                    name_idx = 0;
                }
                this.name = names[name_idx].to_string();
            })),
            ..default()
        }
    },
);
