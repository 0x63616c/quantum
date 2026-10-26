//! Camera gizmo geometry, independent of the window and rendering backend.
use crate::editor::OrbitCamera;
use glam::DVec3;

/// One signed end of a world axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisEnd {
    /// X = 0, Y = 1, Z = 2.
    pub axis: usize,
    /// Whether this is the positive (labelled) end.
    pub positive: bool,
}

impl AxisEnd {
    /// Every clickable end, including the hollow negative ends.
    pub const ALL: [Self; 6] = [
        Self {
            axis: 0,
            positive: false,
        },
        Self {
            axis: 1,
            positive: false,
        },
        Self {
            axis: 2,
            positive: false,
        },
        Self {
            axis: 0,
            positive: true,
        },
        Self {
            axis: 1,
            positive: true,
        },
        Self {
            axis: 2,
            positive: true,
        },
    ];

    /// Unit vector from the target towards the camera for this view.
    pub fn direction(self) -> DVec3 {
        [DVec3::X, DVec3::Y, DVec3::Z][self.axis] * if self.positive { 1.0 } else { -1.0 }
    }

    /// Orient without changing the zoom or panned target. Exact poles use the
    /// camera's yaw-derived right vector, so bottom/top have no singularity.
    pub fn snap(self, camera: &mut OrbitCamera) {
        use std::f64::consts::{FRAC_PI_2, PI};
        (camera.yaw, camera.pitch) = match (self.axis, self.positive) {
            (0, true) => (FRAC_PI_2, 0.0),
            (0, false) => (-FRAC_PI_2, 0.0),
            (1, true) => (0.0, FRAC_PI_2),
            (1, false) => (0.0, -FRAC_PI_2),
            (2, true) => (0.0, 0.0),
            (2, false) => (PI, 0.0),
            _ => unreachable!("axis is X, Y or Z"),
        };
    }
}

/// A projected end in local logical pixels, with camera depth for overlap order.
#[derive(Clone, Copy, Debug)]
pub struct ProjectedEnd {
    /// Signed world axis.
    pub end: AxisEnd,
    /// Position relative to the square gizmo's top-left corner.
    pub position: [f32; 2],
    /// Positive depth is nearer the viewer.
    pub depth: f64,
}

/// Gizmo side length in logical pixels.
pub const SIZE: f32 = 112.0;
/// Endpoint hit radius, also used by the painted circles.
pub const HIT_RADIUS: f32 = 11.0;

/// Back-to-front painter order, also the tie breaker for overlapping targets.
pub fn project(camera: OrbitCamera) -> [ProjectedEnd; 6] {
    let (right, up, back) = camera.basis();
    let mut ends = AxisEnd::ALL.map(|end| {
        let direction = end.direction();
        ProjectedEnd {
            end,
            position: [
                SIZE / 2.0 + direction.dot(right) as f32 * 38.0,
                SIZE / 2.0 - direction.dot(up) as f32 * 38.0,
            ],
            depth: direction.dot(back),
        }
    });
    ends.sort_by(|a, b| a.depth.total_cmp(&b.depth));
    ends
}

/// Pick the frontmost visible circle, never an invisible rectangular corner.
pub fn hit_test(camera: OrbitCamera, point: [f32; 2]) -> Option<AxisEnd> {
    project(camera)
        .into_iter()
        .rev()
        .find(|end| {
            let dx = point[0] - end.position[0];
            let dy = point[1] - end.position[1];
            dx * dx + dy * dy <= HIT_RADIUS * HIT_RADIUS
        })
        .map(|end| end.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Editor;

    fn camera() -> OrbitCamera {
        Editor::blank().unwrap().camera
    }

    #[test]
    fn every_end_snaps_to_the_correct_signed_axis_without_moving_or_zooming() {
        for end in AxisEnd::ALL {
            let mut camera = camera();
            let original = camera;
            end.snap(&mut camera);
            let (right, up, back) = camera.basis();
            assert!(back.abs_diff_eq(end.direction(), 1e-12), "{end:?}");
            assert!((right.dot(up)).abs() < 1e-12);
            assert!((right.length() - 1.0).abs() < 1e-12);
            assert!((up.length() - 1.0).abs() < 1e-12);
            assert_eq!(camera.target, original.target);
            assert_eq!(camera.span, original.span);
            assert!(camera.camera().up.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn picks_all_six_ends_and_rejects_empty_space_and_circle_corners() {
        let camera = camera();
        for projected in project(camera) {
            assert_eq!(hit_test(camera, projected.position), Some(projected.end));
        }
        assert_eq!(hit_test(camera, [0.0, 0.0]), None);
        assert_eq!(hit_test(camera, [SIZE / 2.0; 2]), None);
        assert_eq!(hit_test(camera, [f32::NAN, 0.0]), None);
        let mut front = camera;
        AxisEnd {
            axis: 2,
            positive: true,
        }
        .snap(&mut front);
        assert_eq!(
            hit_test(front, [SIZE / 2.0 + 38.0 + 9.0, SIZE / 2.0 + 9.0]),
            None
        );
    }

    #[test]
    fn overlapping_poles_pick_the_frontmost_end_in_every_axis_view() {
        for end in AxisEnd::ALL {
            let mut camera = camera();
            end.snap(&mut camera);
            assert_eq!(hit_test(camera, [SIZE / 2.0; 2]), Some(end));
        }
    }
}
