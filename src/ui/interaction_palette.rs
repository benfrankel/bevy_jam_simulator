use bevy::prelude::*;

use crate::ui::Disabled;
use crate::AppSet;

pub struct InteractionPalettePlugin;

impl Plugin for InteractionPalettePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InteractionPalette>()
            .add_systems(PostUpdate, update_interaction_color.in_set(AppSet::Animate));
    }
}

// The background color to use for each interaction state
#[derive(Component, Reflect)]
pub struct InteractionPalette {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub disabled: Color,
}

fn update_interaction_color(
    mut interaction_query: Query<
        (
            &Interaction,
            &InteractionPalette,
            Option<&Disabled>,
            &mut BackgroundColor,
        ),
        Or<(Changed<Interaction>, Changed<Disabled>)>,
    >,
) {
    for (interaction, palette, disabled, mut color) in &mut interaction_query {
        color.0 = if matches!(disabled, Some(Disabled(true))) {
            palette.disabled
        } else {
            match interaction {
                Interaction::None => palette.normal,
                Interaction::Hovered => palette.hovered,
                Interaction::Pressed => palette.pressed,
            }
        }
    }
}
