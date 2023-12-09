mod despawn;

use bevy::prelude::*;
use format_num::format_num;
use rand::Rng;

pub use crate::util::despawn::DespawnSet;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(despawn::DespawnPlugin);
    }
}

pub fn pretty_num(x: f64) -> String {
    // See: https://en.wikipedia.org/wiki/Names_of_large_numbers
    const SUFFIXES: [&str; 10] = [
        "million",
        "billion",
        "trillion",
        "quadrillion",
        "quintillion",
        "sextillion",
        "septillion",
        "octillion",
        "nonillion",
        "decillion",
    ];

    let abs = x.abs();
    if abs < 1e9 {
        // Example: 23,480,501
        format_num!(",.0", x)
    } else if abs < 1e36 {
        // Example: 17.012 quadrillion
        let exp_group = abs.log10().floor() as i32 / 3;
        let x = x / (10f64.powi(exp_group * 3));
        let suffix = SUFFIXES[exp_group as usize - 2];
        format!("{} {suffix}", format_num!(".3", x))
    } else if abs < f64::INFINITY {
        // Example: 4.802e65
        format_num!(".3e", x).replace("e+", "e")
    } else {
        "INFINITY".to_string()
    }
}

pub fn gen_color(mut rng: impl Rng) -> Color {
    Color::rgb(
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
    )
}
