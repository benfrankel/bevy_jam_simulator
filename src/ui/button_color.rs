use bevy::prelude::*;

pub struct ButtonColorPlugin;

impl Plugin for ButtonColorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ButtonColor>()
            .add_systems(Update, update_button_color);
    }
}

// The background color to use for each interaction state
#[derive(Component, Reflect)]
pub struct ButtonColor {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn update_button_color(
    mut button_query: Query<
        (&ButtonColor, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (button, interaction, mut color) in &mut button_query {
        color.0 = match interaction {
            Interaction::None => button.normal,
            Interaction::Hovered => button.hovered,
            Interaction::Pressed => button.pressed,
        }
    }
}
