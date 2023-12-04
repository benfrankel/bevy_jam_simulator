use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use crate::config::Config;
use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::vh;
use crate::ui::vw;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub struct TooltipText;

pub fn spawn(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let top_bar_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.top_bar_text_color,
        ..default()
    };
    let tooltip_text_style = TextStyle {
        font: FONT_HANDLE,
        color: config.tooltip_text_color,
        ..default()
    };
    let button_text_style = TextStyle {
        font: FONT_HANDLE,
        color: config.upgrade_button_text_color,
        ..default()
    };

    let system_view = commands
        .spawn((
            Name::new("SystemView"),
            NodeBundle {
                style: Style {
                    width: config.system_view_width,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.system_view_background_color.into(),
                ..default()
            },
            RelativeCursorPosition::default(),
        ))
        .id();

    // Top bar part of the systems view.
    let header_container = commands
        .spawn((
            Name::new("SystemHeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: config.top_bar_height,
                    padding: UiRect::left(Val::VMin(3.5)),
                    border: UiRect::left(config.top_bar_separator_width),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: config.top_bar_background_color.into(),
                border_color: config.top_bar_separator_color.into(),
                ..default()
            },
        ))
        .set_parent(system_view)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section("Systems", top_bar_text_style)
                .with_text_alignment(TextAlignment::Left),
            FontSize::new(config.top_bar_font_size),
        ))
        .set_parent(header_container);

    // Actual content of the upgrade panel.
    let content_container = commands
        .spawn((
            Name::new("UpgradePanel"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.system_view_background_color.into(),
                ..default()
            },
        ))
        .set_parent(system_view)
        .id();

    // Tooltip
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    min_width: Val::Percent(30.0),
                    max_width: Val::Percent(30.0),
                    min_height: Val::Percent(30.0),
                    padding: UiRect::axes(Val::VMin(2.0), Val::VMin(2.0)),
                    margin: UiRect {
                        left: Val::Percent(45.0),
                        top: vh(20.0),
                        right: Val::Percent(25.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                z_index: ZIndex::Global(1000),
                background_color: config.tooltip_background_color.into(),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            Tooltip,
        ))
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section("", tooltip_text_style),
                FontSize::new(config.tooltip_font_size),
                TooltipText,
            ));
        });

    // Buttons
    for _ in 0..4 {
        let button = commands
            .spawn((
                Name::new("UpgradeButton"),
                ButtonBundle {
                    style: Style {
                        margin: UiRect::bottom(vh(4.0)),
                        padding: UiRect::axes(vw(4.0), vh(4.0)),
                        ..default()
                    },
                    background_color: config.upgrade_button_normal_color.into(),
                    ..default()
                },
            ))
            .set_parent(content_container)
            .id();

        commands
            .spawn((
                TextBundle::from_section("10x Dev Upgrade", button_text_style.clone()),
                FontSize::new(config.upgrade_button_font_size),
            ))
            .set_parent(button);
    }

    system_view
}

pub fn interact_with_upgrade_buttons(
    config: Res<Config>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut tooltip: Query<(&mut Visibility, &mut Style), With<Tooltip>>,
    relative_cursor_position_query: Query<&RelativeCursorPosition>,
    mut tooltip_text: Query<&mut Text, With<TooltipText>>,
) {
    let config = &config.editor_screen;
    let (mut tooltip_visibility, mut tooltip_style) = tooltip.single_mut();
    let mut tooltip_text = tooltip_text.single_mut();
    for (interaction, mut color) in &mut interaction_query {
        *color = match interaction {
            Interaction::Pressed => config.upgrade_button_pressed_color,
            Interaction::Hovered => {
                *tooltip_visibility = Visibility::Inherited;
                if let Some(cursor) = relative_cursor_position_query.single().normalized {
                    let percent = (cursor.y * 50.0).min(70.0);
                    tooltip_style.margin.top = Val::Percent(percent);
                }
                tooltip_text.sections[0].value = "This is a tooltip text.".to_string();
                config.upgrade_button_hovered_color
            },
            Interaction::None => {
                *tooltip_visibility = Visibility::Hidden;
                config.upgrade_button_normal_color
            },
        }
        .into()
    }
}
