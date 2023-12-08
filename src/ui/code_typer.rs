use std::iter::Cycle;
use std::str::Chars;

use bevy::prelude::*;

use crate::simulation::Simulation;
use crate::AppSet;

pub struct CodeTyperPlugin;

impl Plugin for CodeTyperPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CodeTyper>()
            .add_systems(Update, type_code.in_set(AppSet::Input));
    }
}

const FILLER_CODE: &str = concat!(
    " to generate lines of code!
// Install the next plugin to start spawning entities.

",
    include_str!("code_typer.rs"),
);

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
            code: default(),
        }
    }
}

impl CodeTyper {
    pub fn enter(&mut self, simulation: &mut Simulation, text: &mut String, count: usize) {
        for _ in 0..count * self.chars_per_key {
            loop {
                // Push a character
                let c = self.code.0.next().unwrap();
                text.push(c);

                // If it was a newline, update typer's lines
                if c == '\n' {
                    simulation.lines += 1.0;
                    self.lines_count += 1;
                    if self.lines_count > self.lines_max {
                        self.lines_count -= 1;
                        // Remove the first line
                        *text = text.split_off(text.find('\n').unwrap() + 1)
                    }
                } else if !c.is_whitespace() {
                    // Stop when a visible character is reached
                    break;
                }
            }
        }
    }
}

pub fn type_code(
    mut char_events: EventReader<ReceivedCharacter>,
    keyboard_input: Res<Input<ScanCode>>,
    mut simulation: ResMut<Simulation>,
    mut typer_query: Query<(&mut CodeTyper, &mut Text)>,
) {
    let count = char_events
        .read()
        .count()
        .min(keyboard_input.get_just_pressed().count());
    if count == 0 {
        return;
    }

    for (mut typer, mut text) in &mut typer_query {
        typer.enter(&mut simulation, &mut text.sections[0].value, count);
    }
}
