use std::iter::Cycle;
use std::str::Chars;

use bevy::prelude::*;

pub struct CodeTyperPlugin;

impl Plugin for CodeTyperPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CodeTyper>()
            .add_systems(Update, type_code);
    }
}

const FILLER_CODE: &str = include_str!("code_typer.rs");

// Newtype so that CodeTyper can derive Reflect
pub struct CodeGenerator(Cycle<Chars<'static>>);

impl Default for CodeGenerator {
    fn default() -> Self {
        Self(FILLER_CODE.chars().cycle())
    }
}

#[derive(Component, Reflect)]
pub struct CodeTyper {
    /// Characters to type per key press.
    pub chars_per_key: usize,
    /// The number of lines currently displayed.
    pub lines_count: usize,
    /// The maximum number of lines to display before old lines start getting deleted.
    pub lines_max: usize,
    /// The total number of \n typed over the full lifetime.
    pub lines_typed: usize,
    /// An infinite iterator that yields the next character that will be added.
    #[reflect(ignore)]
    pub code: CodeGenerator,
}

impl Default for CodeTyper {
    fn default() -> Self {
        Self {
            chars_per_key: 1,
            lines_count: 1,
            lines_max: 1,
            lines_typed: 0,
            code: default(),
        }
    }
}

pub fn type_code(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut CodeTyper, &mut Text)>,
) {
    let count = keyboard_input
        .get_just_pressed()
        .filter(|&&key| KeyCode::Key1 <= key && key <= KeyCode::Z)
        .count();
    if count == 0 {
        return;
    }

    for (mut typer, mut text) in &mut query {
        let text = &mut text.sections[0].value;
        for _ in 0..count * typer.chars_per_key {
            let c = typer.code.0.next().unwrap();
            text.push(c);
            if c != '\n' {
                continue;
            }

            typer.lines_typed += 1;
            typer.lines_count += 1;
            if typer.lines_count > typer.lines_max {
                typer.lines_count -= 1;
                // Remove the first line
                *text = text.split_off(text.find('\n').unwrap() + 1)
            }
        }
    }
}
