//! Orbit camera geometry and bounded voxel picking.
use glam::DVec3;
use quantum_core::*;
use quantum_render::{Camera, OrbitPose, PresetView, Projection};

/// Orthographic orbit rig shared by rendering and picking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    /// World-space look-at point, including pan.
    pub target: DVec3,
    /// Azimuth in radians.
    pub yaw: f64,
    /// Elevation in radians; direct orbit gestures clamp away from the poles.
    pub pitch: f64,
    /// Vertical world-space viewport extent.
    pub span: f64,
    distance: f64,
}
impl OrbitCamera {
    /// Complete pose for interpolation, including the private eye distance.
    pub fn pose(self) -> OrbitPose {
        OrbitPose {
            target: self.target.to_array(),
            yaw: self.yaw,
            pitch: self.pitch,
            span: self.span,
            distance: self.distance,
        }
    }
    /// Restore a pose unchanged, preserving exact top views and endpoints.
    pub fn from_pose(pose: OrbitPose) -> Self {
        Self {
            target: pose.target.into(),
            yaw: pose.yaw,
            pitch: pose.pitch,
            span: pose.span,
            distance: pose.distance,
        }
    }
    /// Fit visible content with a comfortable margin.
    pub fn fit(snapshot: &Snapshot, aspect: f64) -> Self {
        Self::fit_view(snapshot, aspect, PresetView::Iso)
    }
    /// Fit a named orthographic view using its own projected bounds.
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: validated editor document"
    )]
    pub fn fit_view(snapshot: &Snapshot, aspect: f64, view: PresetView) -> Self {
        let camera = Camera::fit(snapshot, view, aspect, false).expect("validated editor document");
        let Projection::Orthographic { height } = camera.projection else {
            unreachable!()
        };
        let mut fitted = Self {
            target: camera.target.into(),
            yaw: match view {
                PresetView::Iso => std::f64::consts::FRAC_PI_4,
                PresetView::Right => std::f64::consts::FRAC_PI_2,
                PresetView::Left => -std::f64::consts::FRAC_PI_2,
                PresetView::Back => std::f64::consts::PI,
                _ => 0.0,
            },
            pitch: match view {
                PresetView::Iso => (1.0 / 3.0_f64.sqrt()).asin(),
                PresetView::Top => std::f64::consts::FRAC_PI_2,
                _ => 0.0,
            },
            span: height * 1.04,
            distance: DVec3::from(camera.eye).distance(camera.target.into()),
        };
        // The center of a world-space bounding box is not necessarily the
        // center of its visible silhouette (for example, a tree beside a house).
        // Project occupied cell extents, retaining the conservative fit scale
        // so sparse scenes do not jump to an unexpectedly tight zoom.
        let (right, up, _) = fitted.basis();
        let half_cell = [right.abs().element_sum(), up.abs().element_sum()].map(|v| v * 0.5);
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for layer in snapshot.layers.iter().filter(|layer| layer.visible) {
            for (position, _) in layer.voxels.iter() {
                let offset =
                    DVec3::from(position.map(f64::from)) + DVec3::splat(0.5) - fitted.target;
                for (axis, value) in [offset.dot(right), offset.dot(up)].into_iter().enumerate() {
                    lo[axis] = lo[axis].min(value - half_cell[axis]);
                    hi[axis] = hi[axis].max(value + half_cell[axis]);
                }
            }
        }
        if lo[0].is_finite() {
            fitted.target += right * ((lo[0] + hi[0]) * 0.5) + up * ((lo[1] + hi[1]) * 0.5);
        }
        fitted
    }
    /// Basis used by both the CPU renderer and screen rays.
    pub fn basis(self) -> (DVec3, DVec3, DVec3) {
        let back = DVec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        );
        // Derive right from yaw so exact top view has no pole singularity.
        let right = DVec3::new(self.yaw.cos(), 0.0, -self.yaw.sin());
        (right, back.cross(right).normalize(), back)
    }
    /// Portable camera vectors for plugin overlay queries.
    pub fn overlay_basis(self) -> [[f64; 3]; 3] {
        let (right, up, back) = self.basis();
        [right.to_array(), up.to_array(), back.to_array()]
    }
    /// Renderer camera; no hidden fit or projection adjustments.
    pub fn camera(self) -> Camera {
        let (_, up, back) = self.basis();
        Camera {
            eye: (self.target + back * self.distance).to_array(),
            target: self.target.to_array(),
            up: up.to_array(),
            projection: Projection::Orthographic { height: self.span },
        }
    }
    /// Orbit by logical pointer pixels.
    pub fn orbit(&mut self, dx: f64, dy: f64) {
        if !dx.is_finite() || !dy.is_finite() {
            return;
        }
        self.yaw = (self.yaw - dx * 0.008).rem_euclid(std::f64::consts::TAU);
        self.pitch = (self.pitch + dy * 0.008).clamp(-1.5, 1.5);
    }
    /// Pan by logical pixels at the current zoom.
    pub fn pan(&mut self, dx: f64, dy: f64, height: f64) {
        if !dx.is_finite() || !dy.is_finite() || !height.is_finite() || height <= 0.0 {
            return;
        }
        let (right, up, _) = self.basis();
        self.target += (-right * dx + up * dy) * self.span / height;
    }
    /// Exponential bounded zoom. Positive wheel delta zooms in.
    pub fn zoom(&mut self, delta: f64) {
        if delta.is_finite() {
            self.span = (self.span * (-delta * 0.002).exp()).clamp(2.0, 2048.0);
        }
    }
    /// Ray through normalized viewport coordinates (left/top = 0, right/bottom = 1).
    pub fn ray(self, uv: [f64; 2], aspect: f64) -> Option<Ray> {
        if !aspect.is_finite()
            || aspect <= 0.0
            || uv
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return None;
        }
        let (right, up, back) = self.basis();
        Some(Ray {
            origin: self.target
                + back * self.distance
                + right * (uv[0] - 0.5) * self.span * aspect
                + up * (0.5 - uv[1]) * self.span,
            direction: -back,
        })
    }
    /// Project a world coordinate to normalized screen coordinates.
    pub fn project(self, p: DVec3, aspect: f64) -> [f64; 2] {
        let (right, up, _) = self.basis();
        let offset = p - self.target;
        [
            0.5 + offset.dot(right) / (self.span * aspect),
            0.5 - offset.dot(up) / self.span,
        ]
    }
}
/// A world-space picking ray.
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    /// Origin before the visible scene.
    pub origin: DVec3,
    /// Nonzero finite direction.
    pub direction: DVec3,
}
/// The nearest visible cell and face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    /// Occupied cell.
    pub at: Pos,
    /// Outward face normal, exactly one signed axis.
    pub normal: Pos,
    /// Topmost visible layer at this cell.
    pub layer: Uuid,
    /// Original stable palette slot.
    pub slot: u16,
}
/// Bounded voxel DDA, including negative coordinates and axis-aligned rays.
pub fn raycast(snapshot: &Snapshot, ray: Ray) -> Option<Hit> {
    if !ray.origin.is_finite()
        || !ray.direction.is_finite()
        || ray.direction.length_squared() < 1e-16
    {
        return None;
    }
    let d = ray.direction.normalize();
    let min = DVec3::from(snapshot.bounds.min.map(f64::from));
    let max = DVec3::from(snapshot.bounds.max.map(f64::from));
    let mut near: f64 = 0.0;
    let mut far = f64::INFINITY;
    let mut normal = [0; 3];
    for a in 0..3 {
        if d[a].abs() < 1e-12 {
            if ray.origin[a] < min[a] || ray.origin[a] >= max[a] {
                return None;
            }
        } else {
            let t1 = (min[a] - ray.origin[a]) / d[a];
            let t2 = (max[a] - ray.origin[a]) / d[a];
            if t1.min(t2) > near {
                near = t1.min(t2);
                normal = [0; 3];
                normal[a] = if d[a] > 0.0 { -1 } else { 1 };
            }
            far = far.min(t1.max(t2));
        }
    }
    if near > far || far < 0.0 {
        return None;
    }
    let start = ray.origin + d * (near + 1e-7);
    let mut cell = start.floor().as_ivec3().to_array();
    let steps = d.to_array().map(|v| if v > 0.0 { 1 } else { -1 });
    let delta = d.to_array().map(|v| {
        if v.abs() < 1e-12 {
            f64::INFINITY
        } else {
            1.0 / v.abs()
        }
    });
    let mut next: [f64; 3] = std::array::from_fn(|a| {
        if d[a].abs() < 1e-12 {
            f64::INFINITY
        } else {
            (f64::from(cell[a] + i32::from(steps[a] > 0)) - ray.origin[a]) / d[a]
        }
    });
    // Each axis is limited to 256 cells by the document contract.
    for _ in 0..772 {
        if !snapshot.bounds.contains(cell) {
            return None;
        }
        if let Some((slot, layer)) = snapshot.sample(cell) {
            return Some(Hit {
                at: cell,
                normal,
                layer,
                slot,
            });
        }
        let a = if next[0] <= next[1] && next[0] <= next[2] {
            0
        } else if next[1] <= next[2] {
            1
        } else {
            2
        };
        if next[a] > far {
            return None;
        }
        cell[a] += steps[a];
        next[a] += delta[a];
        normal = [0; 3];
        normal[a] = -steps[a];
    }
    None
}
