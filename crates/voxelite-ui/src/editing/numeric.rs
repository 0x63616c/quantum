//! Finite single-line numeric input, independent of rendering.

/// Numeric validation, keyboard steps and horizontal scrub policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NumericPolicy {
    /// Inclusive lower bound.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: f64,
    /// Smallest allowed step (also the Option-arrow step).
    pub fine_step: f64,
    /// Whether decimal input is allowed.
    pub decimals: bool,
}
impl NumericPolicy {
    /// An integer range with unit steps.
    pub fn integer(min: f64, max: f64) -> Self {
        let min = if min.is_finite() { min } else { 0. };
        let max = if max.is_finite() { max.max(min) } else { min };
        Self {
            min,
            max,
            fine_step: 1.,
            decimals: false,
        }
    }
    /// A decimal range with a positive fine step.
    pub fn decimal(min: f64, max: f64, fine_step: f64) -> Self {
        Self {
            fine_step: if fine_step.is_finite() && fine_step > 0. {
                fine_step
            } else {
                0.1
            },
            decimals: true,
            ..Self::integer(min, max)
        }
    }
    /// Clamp, quantize and remove negative zero.
    pub fn clamp(self, value: f64) -> f64 {
        let value = if value.is_finite() { value } else { self.min };
        let step = if self.decimals { self.fine_step } else { 1. };
        let result =
            ((value.clamp(self.min, self.max) / step).round() * step).clamp(self.min, self.max);
        if result == 0. { 0. } else { result }
    }
    /// Parse a complete number; incomplete signs and dots are invalid on commit.
    pub fn parse(self, text: &str) -> Option<f64> {
        if text.is_empty() || !self.accepts(text) || matches!(text, "+" | "-" | "." | "+." | "-.") {
            return None;
        }
        text.parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .map(|v| self.clamp(v))
    }
    /// Permit a valid editable prefix, including an unfinished sign or decimal.
    pub fn accepts(self, text: &str) -> bool {
        let mut dot = false;
        text.chars().enumerate().all(|(i, c)| match c {
            '0'..='9' => true,
            '-' | '+' => i == 0 && (c != '-' || self.min < 0.),
            '.' if self.decimals && !dot => {
                dot = true;
                true
            }
            _ => false,
        })
    }
    /// Filter paste against the whole candidate, preserving prefix and suffix.
    pub fn filter(self, before: &str, inserted: &str, after: &str) -> String {
        let mut result = String::new();
        for c in inserted.chars() {
            let candidate = format!("{before}{result}{c}{after}");
            if self.accepts(&candidate) {
                result.push(c);
            }
        }
        result
    }
    /// Step by one, ten with Shift, or the fine step with Option.
    pub fn step(self, value: f64, direction: i8, coarse: bool, fine: bool) -> f64 {
        self.clamp(
            value
                + f64::from(direction)
                    * if fine {
                        self.fine_step
                    } else if coarse {
                        10.
                    } else {
                        1.
                    },
        )
    }
    /// Scrub from the gesture's initial value: four logical pixels per fine step.
    pub fn scrub(self, initial: f64, delta_x: f64, coarse: bool) -> f64 {
        self.clamp(
            initial + (delta_x / 4.).round() * self.fine_step * if coarse { 10. } else { 1. },
        )
    }
    /// Stable decimal display without binary floating point tails.
    pub fn format(self, value: f64) -> String {
        let value = self.clamp(value);
        if !self.decimals {
            return format!("{value:.0}");
        }
        let precision = (0..=9)
            .find(|n| {
                let v = self.fine_step * 10_f64.powi(*n);
                (v - v.round()).abs() < 1e-8
            })
            .unwrap_or(9);
        let s = format!("{value:.precision$}", precision = precision as usize);
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').into()
        } else {
            s
        }
    }
}
