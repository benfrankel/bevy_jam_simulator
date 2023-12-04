use bevy::prelude::*;

pub struct InteractionColorPlugin;

impl Plugin for InteractionColorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InteractionColor>()
            .add_systems(Update, update_button_color);
    }
}

// The background color to use for each interaction state
#[derive(Component, Reflect)]
pub struct InteractionColor {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn update_button_color(
    mut interaction_query: Query<
        (&InteractionColor, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (palette, interaction, mut color) in &mut interaction_query {
        color.0 = match interaction {
            Interaction::None => palette.normal,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        }
    }
}
