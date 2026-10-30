use crate::{RenderError, RenderResult};
use serde::{Deserialize, Serialize};

/// Finite multiplier for local voxel contact shading, in `0.0..=2.0`.
/// Zero disables AO; one preserves the original renderer's shading exactly.
/// Two doubles the darkening, leaving an AO factor of at least 0.44.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct AmbientOcclusionStrength(f64);

impl AmbientOcclusionStrength {
    /// Disable ambient occlusion without changing directional lighting or shadows.
    pub const OFF: Self = Self(0.0);
    /// Largest supported multiplier.
    pub const MAX: f64 = 2.0;

    /// Validate a strength without silently clamping malformed input.
    pub fn new(value: f64) -> RenderResult<Self> {
        if !value.is_finite() || !(0.0..=Self::MAX).contains(&value) {
            return Err(RenderError::Invalid(
                "ambient occlusion strength must be finite and in 0..=2".into(),
            ));
        }
        Ok(Self(value))
    }

    /// Validated multiplier, suitable for a slider or GPU uniform.
    pub fn get(self) -> f64 {
        self.0
    }
}
impl Default for AmbientOcclusionStrength {
    fn default() -> Self {
        Self(1.0)
    }
}
impl TryFrom<f64> for AmbientOcclusionStrength {
    type Error = RenderError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<AmbientOcclusionStrength> for f64 {
    fn from(value: AmbientOcclusionStrength) -> Self {
        value.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strength_is_bounded_and_defaults_to_the_original_look() {
        assert_eq!(AmbientOcclusionStrength::default().get(), 1.0);
        for value in [0.0, 0.5, 1.0, 2.0] {
            let strength = AmbientOcclusionStrength::new(value).unwrap();
            assert_eq!(strength.get(), value);
            assert_eq!(
                serde_json::from_str::<AmbientOcclusionStrength>(
                    &serde_json::to_string(&strength).unwrap()
                )
                .unwrap(),
                strength
            );
        }
        for value in [-0.01, 2.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(AmbientOcclusionStrength::new(value).is_err());
        }
        for json in ["-1", "2.1", "null", "\"1\""] {
            assert!(serde_json::from_str::<AmbientOcclusionStrength>(json).is_err());
        }
    }
}
