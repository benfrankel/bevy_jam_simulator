use std::iter::Cycle;
use std::str::Chars;

use bevy::prelude::*;
use bevy::text::BreakLineOn;

use crate::state::editor_screen::EditorScreenConfig;
use crate::ui::vh;
use crate::ui::FontSize;
use crate::ui::BOLD_FONT_HANDLE;
use crate::ui::FONT_HANDLE;

const CODE_VIEW_START_STRING: &str = "// Press alphanumeric characters randomly to type code.\n\n";
const CODE_STRING: &str = include_str!("code_view.rs");
/// First line in the editor will be removed when the code exceeds this length.
/// TODO: This should be dependent on the screen size.
const CODE_MAX_LENGTH: usize = 600;

/// Component for the text that displays "Lines: X"
#[derive(Component)]
pub struct LinesText;

/// Component for the code text.
#[derive(Component)]
pub struct CodeText;

#[derive(Resource)]
pub struct CodeModel {
    /// Lines of Code.
    loc: f64,
    /// An infinite iterator that yields the next character that will be added to the editor.
    next_code: Cycle<Chars<'static>>,
}

impl Default for CodeModel {
    fn default() -> Self {
        Self {
            loc: 0.0,
            next_code: CODE_STRING.chars().cycle(),
        }
    }
}

pub fn spawn(commands: &mut Commands, config: &EditorScreenConfig) -> Entity {
    let top_bar_text_style = TextStyle {
        font: BOLD_FONT_HANDLE,
        color: config.top_bar_text_color,
        ..default()
    };
    let code_text_style = TextStyle {
        font: FONT_HANDLE,
        color: config.code_text_color,
        ..default()
    };

    commands.init_resource::<CodeModel>();

    let code_view = commands
        .spawn((
            Name::new("CodeView"),
            NodeBundle {
                style: Style {
                    width: config.code_view_width,
                    height: Val::Percent(100.0),
                    // padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    // align_items: AlignItems::Center,
                    // justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.code_view_background_color.into(),
                ..default()
            },
        ))
        .id();

    // Top bar part of the code view.
    let code_header_container = commands
        .spawn((
            Name::new("CodeHeaderContainer"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: vh(20.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    // border: UiRect::bottom(config.top_bar_border_width),
                    ..default()
                },
                background_color: config.top_bar_background_color.into(),
                // border_color: config.top_bar_border_color.into(),
                ..default()
            },
        ))
        .set_parent(code_view)
        .id();

    commands
        .spawn((
            Name::new("HeaderText"),
            TextBundle::from_section("Lines: 0", top_bar_text_style)
                .with_text_alignment(TextAlignment::Left),
            FontSize::new(config.top_bar_font_size),
            LinesText,
        ))
        .set_parent(code_header_container);

    // Actual content of the code view.
    let text_area_container = commands
        .spawn((
            Name::new("TextArea"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::axes(Val::VMin(3.5), Val::VMin(3.5)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: config.code_view_background_color.into(),
                ..default()
            },
        ))
        .set_parent(code_view)
        .id();

    let mut text = TextBundle::from_section(CODE_VIEW_START_STRING, code_text_style)
        .with_text_alignment(TextAlignment::Left);
    text.text.linebreak_behavior = BreakLineOn::AnyCharacter;
    commands
        .spawn((
            Name::new("TextAreaText"),
            text,
            FontSize::new(config.code_font_size),
            CodeText,
        ))
        .set_parent(text_area_container);

    code_view
}

pub fn update_bar(code_model: Res<CodeModel>, mut query: Query<&mut Text, With<LinesText>>) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("Lines: {}", code_model.loc);
}

pub fn type_code(
    mut code_model: ResMut<CodeModel>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Text, With<CodeText>>,
) {
    let mut text = query.single_mut();
    for &key in keyboard_input.get_just_pressed() {
        if KeyCode::Key1 <= key && key <= KeyCode::Z {
            // An alphanumeric key has been pressed.
            code_model.loc += 1.0;
            // Add all whitespace characters at once.
            while {
                let c = code_model.next_code.next().unwrap();
                text.sections[0].value.push(c);
                c.is_whitespace()
            } {}
        }
    }
    // Check if we've exceeded the maximum length.
    if text.sections[0].value.len() > CODE_MAX_LENGTH {
        match text.sections[0].value.find('\n') {
            Some(index) => {
                // Shorten the string by removing the first line.
                text.sections[0].value = text.sections[0].value.split_off(index + 1);
            },
            None => {
                // There's no line, just clear the string (keeps capacity).
                text.sections[0].value.clear();
            },
        };
    }
}
