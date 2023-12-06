mod despawn;

use bevy::prelude::*;

pub use crate::util::despawn::DespawnSet;
pub use crate::util::despawn::OverflowDespawnQueue;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(despawn::DespawnPlugin);
    }
}
