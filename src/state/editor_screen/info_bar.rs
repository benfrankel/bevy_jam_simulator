use bevy::prelude::*;

use super::EditorScreenTheme;
use crate::simulation::Simulation;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;

pub struct InfoBarPlugin;

impl Plugin for InfoBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InfoBarText>()
            .add_systems(Update, update_info_bar_text);
    }
}

pub fn spawn_info_bar(commands: &mut Commands, theme: &EditorScreenTheme) -> Entity {
    let info_bar = commands
        .spawn((
            Name::new("InfoBar"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    min_height: theme.info_bar_height,
                    padding: UiRect::horizontal(Val::Px(16.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: theme.info_bar_background_color.into(),
                ..default()
            },
        ))
        .id();

    commands
        .spawn((
            Name::new("InfoBarText"),
            TextBundle::from_section(
                "",
                TextStyle {
                    font: BOLD_FONT_HANDLE,
                    color: theme.info_bar_text_color,
                    ..default()
                },
            ),
            FontSize::new(theme.info_bar_font_size),
            InfoBarText,
        ))
        .set_parent(info_bar);

    info_bar
}

#[derive(Component, Reflect)]
struct InfoBarText;

fn update_info_bar_text(
    simulation: Res<Simulation>,
    mut info_bar_query: Query<&mut Text, With<InfoBarText>>,
) {
    // TODO: E.g. Format large numbers like 2,346,834 and then 8.435e22
    let lines = simulation.lines;
    let entities = simulation.entities;

    // TODO: Remove "s" if number is equal to 1
    let info = format!("{lines} lines, {entities} entities");

    for mut text in &mut info_bar_query {
        text.sections[0].value = info.clone();
    }
}
