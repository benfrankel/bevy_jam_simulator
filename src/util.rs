mod despawn;

use bevy::prelude::*;
use format_num::format_num;

pub use crate::util::despawn::DespawnSet;
pub use crate::util::despawn::OverflowDespawnQueue;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(despawn::DespawnPlugin);
    }
}

pub fn pretty_num(x: f64) -> String {
    let abs = x.abs();
    if abs < 1e9 {
        format_num!(",.0", x)
    } else if abs < 1e18 {
        // TODO: e.g. "123.456 billion", "8.012 quadrillion" instead of scientific notation
        format_num!(".3e", x).replace("e+", "e")
    } else if abs < f64::INFINITY {
        format_num!(".3e", x).replace("e+", "e")
    } else {
        "INFINITY".to_string()
    }
}
