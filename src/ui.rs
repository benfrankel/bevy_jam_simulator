mod code_typer;
mod font;
mod interaction_palette;
mod scroll;
mod tooltip;

use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

pub use crate::ui::code_typer::CodeTyper;
pub use crate::ui::font::FontSize;
pub use crate::ui::font::BOLD_FONT_HANDLE;
pub use crate::ui::font::FONT_HANDLE;
pub use crate::ui::font::HEADER_FONT_HANDLE;
pub use crate::ui::interaction_palette::InteractionPalette;
pub use crate::ui::scroll::ScrollContent;
pub use crate::ui::tooltip::Tooltip;
pub use crate::ui::tooltip::TooltipConfig;
pub use crate::ui::tooltip::TooltipSide;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Disabled>().add_plugins((
            DefaultPickingPlugins,
            code_typer::CodeTyperPlugin,
            font::FontPlugin,
            interaction_palette::InteractionPalettePlugin,
            scroll::ScrollPlugin,
            tooltip::TooltipPlugin,
        ));
    }
}

#[derive(Component, Reflect)]
pub struct Disabled(pub bool);
