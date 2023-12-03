mod font;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::camera::CAMERA_HEIGHT;
use crate::camera::CAMERA_WIDTH;
pub use crate::ui::font::FontSize;
pub use crate::ui::font::BOLD_FONT_HANDLE;
pub use crate::ui::font::FONT_HANDLE;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPickingPlugins, font::FontPlugin));
    }
}

pub fn vw(units: f32) -> Val {
    Val::Vw(100.0 / CAMERA_WIDTH * units)
}

pub fn vh(units: f32) -> Val {
    Val::Vh(100.0 / CAMERA_HEIGHT * units)
}

pub fn vmin(units: f32) -> Val {
    Val::VMin(100.0 / CAMERA_HEIGHT * units)
}
