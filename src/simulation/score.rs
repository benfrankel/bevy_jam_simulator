//! Module responsible for score calculation.
use super::*;

/// - x: Input
/// - l: Max value
/// - k: Growth rate
/// - x_0: Midpoint
fn logistic_function(x: f64, l: f64, k: f64, x_0: f64) -> f64 {
    l / (1.0 + (-k * (x - x_0)).exp())
}

/// Calculate score using a logistic function based on the given upper and lower values.
/// When x = upper, the output will be approximately 4.620.
/// When x = lower, the output will be approximately 0.379.
fn calculate_score(x: f64, lower: f64, upper: f64) -> f64 {
    let k = 5.0 / (upper - lower);
    let x_0 = (upper + lower) / 2.0;
    logistic_function(x, 5.0, k, x_0)
}

impl Simulation {
    /// Returns the calculated score (between 0 and 5) for each category.
    /// 1. Fun
    /// 2. Presentation
    /// 3. Theme Interpretation
    /// 4. Entities
    /// 5. Lines of Code
    /// 6. Overall
    pub fn calculate_scores(&self) -> [f64; 6] {
        let mut scores: [f64; 6] = [
            // Fun
            calculate_score(self.fun_factor, 0.0, 70.0),
            // Presentation
            calculate_score(self.presentation_factor, 0.0, 40.0),
            // Theme Interpretation
            calculate_score(self.entities, 0.0, 1e9),
            // Entities
            calculate_score(self.entities, 0.0, 1e9),
            // Lines of Code
            calculate_score(self.lines, 0.0, 1e9),
            // Overall
            0.0,
        ];
        scores[5] = scores.iter().sum::<f64>() / 5.0;
        scores
    }
}
