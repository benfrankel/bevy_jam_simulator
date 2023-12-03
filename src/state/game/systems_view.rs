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

pub fn init(commands: &mut Commands, root: &Res<AppRoot>) {
    let code_view = commands
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
        .set_parent(code_view)
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
        .set_parent(code_view)
        .id();

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
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match interaction {
            Interaction::Pressed => BUTTON_PRESSED_COLOR,
            Interaction::Hovered => BUTTON_HOVERED_COLOR,
            Interaction::None => BUTTON_NORMAL_COLOR,
        }
        .into()
    }
}
