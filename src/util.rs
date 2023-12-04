mod code_typer;

use bevy::prelude::*;
pub use code_typer::CodeTyper;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(code_typer::CodeTyperPlugin);
    }
}
