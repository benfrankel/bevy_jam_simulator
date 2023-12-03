use super::*;
use crate::ui::vh;
use crate::ui::FontSize;

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

pub fn init(commands: &mut Commands, root: &Res<AppRoot>) {
    commands.insert_resource(EntityModel::default());

    // Top bar part of the code view.
    let header_container = commands
        .spawn((
            Name::new("EntityHeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(ENTITY_VIEW_WIDTH),
                    height: vh(20.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    border: UiRect::left(vh(TOP_BAR_SEPARATOR_WIDTH)),
                    ..default()
                },
                background_color: TOP_BAR_BACKGROUND_COLOR.into(),
                border_color: TOP_BAR_SEPARATOR_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(root.ui)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section("Entities: 0", TOP_BAR_TEXT_STYLE)
                .with_text_alignment(TextAlignment::Left),
            FontSize::new(vh(TOP_BAR_FONT_SIZE)),
            EntitiesText,
        ))
        .set_parent(header_container);
}

pub fn update_bar(
    mut entity_model: ResMut<EntityModel>,
    mut query: Query<&mut Text, With<EntitiesText>>,
) {
    let mut text = query.single_mut();
    entity_model.count += 1.0;
    text.sections[0].value = format!("Entities: {}", entity_model.count);
}
