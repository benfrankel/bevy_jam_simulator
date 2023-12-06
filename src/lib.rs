#![allow(clippy::type_complexity)]

mod audio;
mod camera;
mod config;
#[cfg(feature = "dev")]
mod debug;
mod physics;
mod simulation;
mod state;
mod ui;
mod upgrade;
mod util;

use bevy::log::LogPlugin;
use bevy::prelude::*;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AppRoot>()
            .init_resource::<AppRoot>()
            .add_systems(Startup, spawn_logical_entities);

        // System ordering
        app.configure_sets(
            Update,
            (
                AppSet::Start,
                AppSet::Tick,
                AppSet::Input,
                AppSet::RunUpgrades,
                AppSet::Simulate,
                AppSet::Update,
                AppSet::Despawn,
                AppSet::ApplyDeferred,
                AppSet::End,
            )
                .chain(),
        )
        .add_systems(Update, apply_deferred.in_set(AppSet::ApplyDeferred));

        // Order-dependent plugins
        app.add_plugins((
            LogPlugin::default(),
            config::ConfigPlugin,
            DefaultPlugins
                .build()
                .disable::<LogPlugin>()
                .disable::<WindowPlugin>()
                .set(ImagePlugin::default_nearest()),
            audio::AudioPlugin,
            state::StatePlugin,
        ));

        // Other plugins
        app.add_plugins((
            camera::CameraPlugin,
            physics::PhysicsPlugin,
            simulation::SimulationPlugin,
            ui::UiPlugin,
            upgrade::UpgradePlugin,
            util::UtilPlugin,
        ));

        #[cfg(feature = "dev")]
        app.add_plugins(debug::DebugPlugin {
            ambiguity_detection: false,
            //editor: false,
            start: state::AppState::EditorScreen,
            ..default()
        });
    }
}

#[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AppSet {
    /// Initialize start-of-frame values
    Start,
    /// Tick timers
    Tick,
    /// Handle input
    Input,
    /// Enable / update upgrades
    RunUpgrades,
    /// Step the simulation
    Simulate,
    /// Update everything else
    Update,
    /// Queue despawn commands
    Despawn,
    /// Apply deferred (commands)
    ApplyDeferred,
    /// Synchronize end-of-frame values (after commands have been applied)
    End,
}

// Global entities
#[derive(Resource, Reflect)]
pub struct AppRoot {
    window: Entity,
    camera: Entity,
    tooltip: Entity,
    tooltip_text: Entity,

    // Logical entities
    ui: Entity,
    world: Entity,
}

impl Default for AppRoot {
    fn default() -> Self {
        Self {
            window: Entity::PLACEHOLDER,
            camera: Entity::PLACEHOLDER,
            tooltip: Entity::PLACEHOLDER,
            tooltip_text: Entity::PLACEHOLDER,

            ui: Entity::PLACEHOLDER,
            world: Entity::PLACEHOLDER,
        }
    }
}

fn spawn_logical_entities(mut commands: Commands, mut root: ResMut<AppRoot>) {
    root.ui = commands
        .spawn((
            Name::new("Ui"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    root.world = commands
        .spawn((Name::new("World"), SpatialBundle::default()))
        .id();
}
