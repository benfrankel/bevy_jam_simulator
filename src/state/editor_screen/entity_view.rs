use bevy::prelude::*;

use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::vh;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;

#[derive(Resource)]
pub struct EntityModel {
    /// Number of entities.
    count: f64,
}

impl Default for EntityModel {
    fn default() -> Self {
        Self { count: 0.0 }
    }
}

/// Component for the text that displays "Entities: X"
#[derive(Component)]
pub struct EntitiesText;

pub fn spawn(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let top_bar_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.top_bar_text_color,
        ..default()
    };

    commands.init_resource::<EntityModel>();

    let entity_view = commands
        .spawn((
            Name::new("EntityView"),
            NodeBundle {
                style: Style {
                    width: config.entity_view_width,
                    height: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    // Top bar part of the code view.
    let header_container = commands
        .spawn((
            Name::new("EntityHeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: vh(20.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    border: UiRect::left(config.top_bar_separator_width),
                    ..default()
                },
                background_color: config.top_bar_background_color.into(),
                border_color: config.top_bar_separator_color.into(),
                ..default()
            },
        ))
        .set_parent(entity_view)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section("Entities: 0", top_bar_text_style)
                .with_text_alignment(TextAlignment::Left),
            FontSize::new(config.top_bar_font_size),
            EntitiesText,
        ))
        .set_parent(header_container);

    entity_view
}

pub fn update_bar(
    mut entity_model: ResMut<EntityModel>,
    mut query: Query<&mut Text, With<EntitiesText>>,
) {
    let mut text = query.single_mut();
    entity_model.count += 1.0;
    text.sections[0].value = format!("Entities: {}", entity_model.count);
}
