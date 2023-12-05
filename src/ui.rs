mod code_typer;
mod font;
mod interaction_palette;
mod tooltip;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

use crate::camera::CAMERA_HEIGHT;
use crate::camera::CAMERA_WIDTH;
pub use crate::ui::code_typer::CodeTyper;
pub use crate::ui::font::FontSize;
pub use crate::ui::font::BOLD_FONT_HANDLE;
pub use crate::ui::font::FONT_HANDLE;
pub use crate::ui::interaction_palette::InteractionPalette;
pub use crate::ui::tooltip::Tooltip;
pub use crate::ui::tooltip::TooltipConfig;
pub use crate::ui::tooltip::TooltipSide;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Disabled>().add_plugins((
            DefaultPickingPlugins,
            interaction_palette::InteractionPalettePlugin,
            code_typer::CodeTyperPlugin,
            font::FontPlugin,
            tooltip::TooltipPlugin,
        ));
    }
}

#[derive(Component, Reflect)]
pub struct Disabled(pub bool);

pub fn vw(units: f32) -> Val {
    Val::Vw(100.0 / CAMERA_WIDTH * units)
}

pub fn vh(units: f32) -> Val {
    Val::Vh(100.0 / CAMERA_HEIGHT * units)
}

pub fn vmin(units: f32) -> Val {
    Val::VMin(100.0 / CAMERA_HEIGHT * units)
}
