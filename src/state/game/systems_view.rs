use bevy::ui::RelativeCursorPosition;

use super::*;
use crate::ui::vh;
use crate::ui::vw;
use crate::ui::FontSize;
use crate::ui::FONT_HANDLE;

const BACKGROUND_COLOR: Color = Color::rgb(0.106, 0.118, 0.122);

const BUTTON_TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: 0.0,
    color: Color::WHITE,
};
const BUTTON_FONT_SIZE: f32 = 4.0;
const BUTTON_NORMAL_COLOR: Color = Color::rgb(0.165, 0.18, 0.184);
const BUTTON_HOVERED_COLOR: Color = Color::rgb(0.265, 0.28, 0.284);
const BUTTON_PRESSED_COLOR: Color = Color::rgb(0.065, 0.08, 0.084);

const TOOLTIP_TEXT_STYLE: TextStyle = TextStyle {
    font: FONT_HANDLE,
    font_size: 0.0,
    color: Color::WHITE,
};
const TOOLTIP_FONT_SIZE: f32 = 4.0;
const TOOLTIP_BACKGROUND_COLOR: Color = Color::rgba(0.106, 0.118, 0.122, 0.75);

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub struct TooltipText;

pub fn init(commands: &mut Commands, root: &Res<AppRoot>) {
    let systems_view = commands
        .spawn((
            Name::new("SystemsView"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(SYSTEMS_VIEW_WIDTH),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BACKGROUND_COLOR.into(),
                ..default()
            },
        ))
        .insert(RelativeCursorPosition::default())
        .set_parent(root.ui)
        .id();

    // Top bar part of the systems view.
    let header_container = commands
        .spawn((
            Name::new("SystemHeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
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
        .set_parent(systems_view)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section("Upgrades", TOP_BAR_TEXT_STYLE)
                .with_text_alignment(TextAlignment::Left),
            FontSize::new(vh(TOP_BAR_FONT_SIZE)),
        ))
        .set_parent(header_container);

    // Actual content of the upgrades panel.
    let content_container = commands
        .spawn((
            Name::new("UpgradesPanel"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: BACKGROUND_COLOR.into(),
                ..default()
            },
        ))
        .set_parent(systems_view)
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
                background_color: TOOLTIP_BACKGROUND_COLOR.into(),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            Tooltip,
        ))
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section("", TOOLTIP_TEXT_STYLE),
                FontSize::new(vh(TOOLTIP_FONT_SIZE)),
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
                    background_color: BUTTON_NORMAL_COLOR.into(),
                    ..default()
                },
            ))
            .set_parent(content_container)
            .id();

        commands
            .spawn((
                TextBundle::from_section("10x Dev Upgrade", BUTTON_TEXT_STYLE),
                FontSize::new(vh(BUTTON_FONT_SIZE)),
            ))
            .set_parent(button);
    }
}

pub fn button_color_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut tooltip: Query<(&mut Visibility, &mut Style), With<Tooltip>>,
    relative_cursor_position_query: Query<&RelativeCursorPosition>,
    mut tooltip_text: Query<&mut Text, With<TooltipText>>,
) {
    let (mut tooltip_visibility, mut tooltip_style) = tooltip.single_mut();
    let mut tooltip_text = tooltip_text.single_mut();
    for (interaction, mut color) in &mut interaction_query {
        *color = match interaction {
            Interaction::Pressed => BUTTON_PRESSED_COLOR,
            Interaction::Hovered => {
                *tooltip_visibility = Visibility::Inherited;
                if let Some(cursor) = relative_cursor_position_query.single().normalized {
                    let percent = (cursor.y * 50.0).min(70.0);
                    tooltip_style.margin.top = Val::Percent(percent);
                }
                tooltip_text.sections[0].value = "This is a tooltip text.".to_string();
                BUTTON_HOVERED_COLOR
            },
            Interaction::None => {
                *tooltip_visibility = Visibility::Hidden;
                BUTTON_NORMAL_COLOR
            },
        }
        .into()
    }
}
