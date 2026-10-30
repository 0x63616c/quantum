use crate::{RenderError, RenderResult};
use glam::DVec3;
use voxelite_core::{Region, Snapshot};

/// Standard Y-up camera directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetView {
    /// View from normalized [1,1,1].
    Iso,
    /// Look from +Z.
    Front,
    /// Look from −Z.
    Back,
    /// Look from +X.
    Right,
    /// Look from −X.
    Left,
    /// Look from +Y with −Z up.
    Top,
}
/// Projection with parameters that cannot be combined ambiguously.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    /// Vertical world-space span, strictly positive.
    Orthographic {
        /// Vertical span.
        height: f64,
    },
    /// Vertical field of view in degrees, strictly between 1 and 179.
    Perspective {
        /// Vertical field of view.
        fov_y_degrees: f64,
    },
}
/// Explicit camera, independent of any human viewport state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// Finite eye position.
    pub eye: [f64; 3],
    /// Finite look-at target, distinct from eye.
    pub target: [f64; 3],
    /// Nonzero up direction, not parallel to the look direction.
    pub up: [f64; 3],
    /// Projection and its one applicable parameter.
    pub projection: Projection,
}
impl Camera {
    /// Validate finite, nondegenerate camera inputs before allocating targets.
    pub fn validate(self) -> RenderResult<()> {
        if self
            .eye
            .iter()
            .chain(&self.target)
            .chain(&self.up)
            .any(|v| !v.is_finite() || v.abs() > 1e9)
        {
            return Err(RenderError::Invalid(
                "camera components must be finite and within ±1e9".into(),
            ));
        }
        let direction = DVec3::from(self.target) - DVec3::from(self.eye);
        let up = DVec3::from(self.up);
        if direction.length_squared() < 1e-16
            || up.length_squared() < 1e-16
            || direction.normalize().cross(up.normalize()).length_squared() < 1e-12
        {
            return Err(RenderError::Invalid(
                "camera eye/target/up are degenerate".into(),
            ));
        }
        match self.projection {
            Projection::Orthographic { height }
                if !height.is_finite() || height <= 0.0 || height > 1e9 =>
            {
                return Err(RenderError::Invalid(
                    "orthographic height must be finite in (0,1e9]".into(),
                ));
            }
            Projection::Perspective { fov_y_degrees }
                if !fov_y_degrees.is_finite() || fov_y_degrees <= 1.0 || fov_y_degrees >= 179.0 =>
            {
                return Err(RenderError::Invalid(
                    "perspective FOV must be in (1,179)".into(),
                ));
            }
            _ => {}
        }
        Ok(())
    }
    /// Fit visible content with a 10% margin. Empty scenes use document bounds.
    pub fn fit(
        snapshot: &Snapshot,
        preset: PresetView,
        aspect: f64,
        perspective: bool,
    ) -> RenderResult<Self> {
        snapshot.validate()?;
        if !aspect.is_finite() || aspect <= 0.0 {
            return Err(RenderError::Invalid("aspect ratio".into()));
        }
        let bounds = visible_bounds(snapshot);
        let min = DVec3::from(bounds.min.map(f64::from));
        let max = DVec3::from(bounds.max.map(f64::from));
        let target = (min + max) * 0.5;
        let radius = (max - min).length() * 0.5;
        let direction = match preset {
            PresetView::Iso => DVec3::ONE.normalize(),
            PresetView::Front => DVec3::Z,
            PresetView::Back => DVec3::NEG_Z,
            PresetView::Right => DVec3::X,
            PresetView::Left => DVec3::NEG_X,
            PresetView::Top => DVec3::Y,
        };
        let up = if preset == PresetView::Top {
            DVec3::NEG_Z
        } else {
            DVec3::Y
        };
        let right = (-direction).cross(up).normalize();
        let up = right.cross(-direction).normalize();
        let mut span: f64 = 0.0;
        for p in corners(bounds) {
            let offset = p - target;
            span = span
                .max(offset.dot(up).abs())
                .max(offset.dot(right).abs() / aspect);
        }
        let (distance, projection) = if perspective {
            let angle = 22.5f64
                .to_radians()
                .tan()
                .min(22.5f64.to_radians().tan() * aspect)
                .atan();
            (
                radius / angle.sin() * 1.1,
                Projection::Perspective {
                    fov_y_degrees: 45.0,
                },
            )
        } else {
            (
                radius * 3.0 + 1.0,
                Projection::Orthographic { height: span * 2.2 },
            )
        };
        let camera = Self {
            eye: (target + direction * distance).to_array(),
            target: target.to_array(),
            up: up.to_array(),
            projection,
        };
        camera.validate()?;
        Ok(camera)
    }
}
/// Occupied visible bounds, or document bounds when empty.
pub fn visible_bounds(snapshot: &Snapshot) -> Region {
    let mut bounds: Option<Region> = None;
    for layer in snapshot.layers.iter().filter(|l| l.visible) {
        for (p, _) in layer.voxels.iter() {
            if let Some(r) = &mut bounds {
                for (a, c) in p.iter().enumerate() {
                    r.min[a] = r.min[a].min(*c);
                    r.max[a] = r.max[a].max(*c + 1);
                }
            } else {
                bounds = Some(Region {
                    min: p,
                    max: p.map(|v| v + 1),
                });
            }
        }
    }
    bounds.unwrap_or(snapshot.bounds)
}
pub(crate) fn corners(bounds: Region) -> impl Iterator<Item = DVec3> {
    (0..8).map(move |bits| {
        DVec3::from(std::array::from_fn(|a| {
            f64::from(if bits & (1 << a) == 0 {
                bounds.min[a]
            } else {
                bounds.max[a]
            })
        }))
    })
}
