//! GPU-only directional shadow fitting, kept independent of wgpu for host tests.
use glam::{DMat4, DVec3, DVec4, Mat4, Vec3};
use voxelite_core::Region;

pub(crate) const SHADOW_SIZE: u32 = 4096;
// Reserve texels for the 3x3 bilinear PCF footprint and snapping.
const GUARD_TEXELS: f64 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ShadowInput {
    pub bounds: Region,
    pub view_projection: Mat4,
    pub light: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ShadowFit {
    pub projection: Mat4,
    pub normal_offset: f32,
}

#[derive(Default)]
pub(crate) struct ShadowCache {
    input: Option<ShadowInput>,
    pub fit: Option<ShadowFit>,
}

impl ShadowCache {
    /// Ambient frames skip depth rendering. Discard the fit so re-enabling always
    /// redraws, including after edits made while lighting was disabled.
    pub fn update_lighting(
        &mut self,
        input: ShadowInput,
        scene_changed: bool,
        lighting: crate::Lighting,
    ) -> bool {
        if lighting == crate::Lighting::Ambient {
            *self = Self::default();
            false
        } else {
            self.update(input, scene_changed)
        }
    }
    /// Style-only frames reuse both the fit and the depth texture. Scene edits
    /// (including deleted chunks) invalidate depth even with identical bounds.
    pub fn update(&mut self, input: ShadowInput, scene_changed: bool) -> bool {
        let view_changed = self.input != Some(input);
        if view_changed {
            self.fit = fit_shadow(input);
            self.input = Some(input);
        }
        self.fit.is_some() && (view_changed || scene_changed)
    }
}

fn fit_shadow(input: ShadowInput) -> Option<ShadowFit> {
    let receivers = visible_vertices(input.bounds, input.view_projection);
    if receivers.is_empty() {
        return None;
    }
    let light = input.light.as_dvec3().normalize();
    // A world-anchored light basis is essential: snapping a camera-relative
    // center would still let the shadow texels slide across stationary geometry.
    let rotation = DMat4::look_at_rh(
        DVec3::ZERO,
        -light,
        if light.dot(DVec3::Y).abs() > 0.99 {
            DVec3::Z
        } else {
            DVec3::Y
        },
    );
    let mut min = DVec3::splat(f64::INFINITY);
    let mut max = DVec3::splat(f64::NEG_INFINITY);
    for vertex in receivers {
        let p = rotation.transform_point3(vertex);
        min = min.min(p);
        max = max.max(p);
    }
    // Shrinks with the fit at close zoom; cap at 5% of a voxel at overview.
    let normal_offset =
        ((max - min).truncate().max_element() * 0.75 / f64::from(SHADOW_SIZE)).min(0.05);
    // Pad in world units before snapping. In a very narrow fit, an offset based
    // on the larger axis can cover many texels of the smaller axis.
    let (x, width, _) = snap_axis(min.x - normal_offset, max.x + normal_offset);
    let (y, height, _) = snap_axis(min.y - normal_offset, max.y + normal_offset);

    // Crop X/Y to receivers, but keep the whole scene's light-space depth.
    // Offscreen casters along the same light ray must not be near-plane clipped.
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for vertex in crate::camera::corners(input.bounds) {
        let z = rotation.transform_point3(vertex).z;
        z_min = z_min.min(z);
        z_max = z_max.max(z);
    }
    let projection = DMat4::orthographic_rh(
        x - width * 0.5,
        x + width * 0.5,
        y - height * 0.5,
        y + height * 0.5,
        -z_max - 1.0,
        -z_min + 1.0,
    ) * rotation;
    Some(ShadowFit {
        projection: projection.as_mat4(),
        normal_offset: normal_offset as f32,
    })
}

fn snap_axis(min: f64, max: f64) -> (f64, f64, f64) {
    // Quantize the extent upward in small logarithmic steps (~2.2%). This keeps
    // the texel grid's scale stable through small orbit/fit changes at any zoom.
    // The tiny minimum also handles a frustum merely touching a scene face.
    let span = ((max - min).max(0.001).log2() * 32.0).ceil() / 32.0;
    let texel = span.exp2() / (f64::from(SHADOW_SIZE) - 2.0 * GUARD_TEXELS);
    let center = ((min + max) * 0.5 / texel).round() * texel;
    (center, texel * f64::from(SHADOW_SIZE), texel)
}

/// Vertices of the actual convex intersection, not an intersection of world
/// AABBs (which wastes texels on oblique views). Every vertex lies on three of
/// the twelve planes. This also finds edge/face crossings when neither volume
/// has a corner inside the other. At most 220 plane triples, independent of mesh.
fn visible_vertices(bounds: Region, view_projection: Mat4) -> Vec<DVec3> {
    // Use f64 for intersections, especially the almost parallel near/far planes
    // of perspective views, but derive them from the exact matrix sent to wgpu.
    let matrix = view_projection.as_dmat4();
    let [x, y, z, w] = std::array::from_fn(|i| matrix.row(i));
    // wgpu clip space: -w <= x,y <= w; 0 <= z <= w.
    let mut planes = [DVec4::ZERO; 12];
    for (plane, p) in planes
        .iter_mut()
        .zip([w + x, w - x, w + y, w - y, z, w - z])
    {
        *plane = p / p.truncate().length();
    }
    for axis in 0..3 {
        let mut normal = DVec3::ZERO;
        normal[axis] = 1.0;
        planes[6 + axis * 2] = normal.extend(-f64::from(bounds.min[axis]));
        planes[7 + axis * 2] = (-normal).extend(f64::from(bounds.max[axis]));
    }
    let mut vertices = Vec::new();
    for (i, a) in planes.iter().enumerate() {
        for (j, b) in planes.iter().enumerate().skip(i + 1) {
            for c in planes.iter().skip(j + 1) {
                let bc = b.truncate().cross(c.truncate());
                let determinant = a.truncate().dot(bc);
                if determinant.abs() < 1e-10 {
                    continue;
                }
                let p = -(a.w * bc
                    + b.w * c.truncate().cross(a.truncate())
                    + c.w * a.truncate().cross(b.truncate()))
                    / determinant;
                if p.is_finite() && planes.iter().all(|plane| plane.dot(p.extend(1.0)) >= -1e-6) {
                    vertices.push(p);
                }
            }
        }
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(height: f32) -> ShadowInput {
        ShadowInput {
            bounds: Region::new([-16; 3], [16; 3]).unwrap(),
            view_projection: Mat4::orthographic_rh(
                -height * 0.5,
                height * 0.5,
                -height * 0.5,
                height * 0.5,
                1.0,
                64.0,
            ) * Mat4::look_at_rh(Vec3::Z * 32.0, Vec3::ZERO, Vec3::Y),
            light: Vec3::Z,
        }
    }

    fn assert_covered(projection: Mat4, point: Vec3) {
        let p = projection.project_point3(point);
        assert!(
            p.x.abs() <= 1.0001 && p.y.abs() <= 1.0001,
            "{point:?} -> {p:?}"
        );
        assert!((-0.0001..=1.0001).contains(&p.z), "{point:?} -> {p:?}");
    }

    #[test]
    fn disabled_lighting_invalidates_depth_until_reenabled() {
        let mut cache = ShadowCache::default();
        let input = input(32.0);
        assert!(cache.update_lighting(input, false, crate::Lighting::Exact));
        assert!(!cache.update_lighting(input, false, crate::Lighting::Exact));
        assert!(!cache.update_lighting(input, true, crate::Lighting::Ambient));
        assert!(cache.fit.is_none());
        assert!(!cache.update_lighting(input, false, crate::Lighting::Ambient));
        // No new geometry update is sent on this style-only frame.
        assert!(cache.update_lighting(input, false, crate::Lighting::Exact));
        assert!(!cache.update_lighting(input, false, crate::Lighting::Exact));
    }

    #[test]
    fn fits_edge_face_intersections_without_contained_corners() {
        let input = input(4.0);
        let vertices = visible_vertices(input.bounds, input.view_projection);
        assert_eq!(vertices.len(), 8);
        for p in vertices {
            assert!((p.x.abs() - 2.0).abs() < 1e-5);
            assert!((p.y.abs() - 2.0).abs() < 1e-5);
            assert!((p.z.abs() - 16.0).abs() < 1e-5);
            assert_covered(fit_shadow(input).unwrap().projection, p.as_vec3());
        }
    }

    #[test]
    fn perspective_intersection_obeys_near_far_and_scene_planes() {
        let bounds = Region::new([-10; 3], [10; 3]).unwrap();
        let matrix = Mat4::perspective_rh(90.0_f32.to_radians(), 2.0, 1.0, 8.0);
        let vertices = visible_vertices(bounds, matrix);
        for expected in [
            DVec3::new(-2.0, -1.0, -1.0),
            DVec3::new(2.0, 1.0, -1.0),
            DVec3::new(10.0, 8.0, -8.0),
            DVec3::new(-10.0, -8.0, -8.0),
        ] {
            assert!(vertices.iter().any(|p| p.distance(expected) < 1e-4));
        }
        for p in vertices {
            assert!((-8.0001..=-0.9999).contains(&p.z));
            assert!(p.x.abs() <= 10.0001 && p.y.abs() <= 10.0001);
        }
    }

    #[test]
    fn zoom_increases_density_and_reduces_normal_bias() {
        let wide = fit_shadow(input(32.0)).unwrap();
        let close = fit_shadow(input(2.0)).unwrap();
        assert!(close.projection.x_axis.x > wide.projection.x_axis.x * 15.9);
        assert!(close.normal_offset < wide.normal_offset / 15.9);
    }

    #[test]
    fn snapping_is_world_anchored_and_preserves_filter_guard() {
        let (center, width, texel) = snap_axis(-2.6, 2.6);
        let shifted = snap_axis(-2.6 + texel * 0.2, 2.6 + texel * 0.2);
        assert_eq!((center, width, texel), shifted);
        let (next, _, _) = snap_axis(-2.6 + texel * 0.6, 2.6 + texel * 0.6);
        assert!((next - center - texel).abs() < 1e-12);
        assert_eq!(snap_axis(-2.60001, 2.60001).1, width);
        for offset in [-32700.0, -1.234, 0.0, 1.234, 32700.0] {
            let (center, width, texel) = snap_axis(offset - 2.6, offset + 2.6);
            assert!((center / texel - (center / texel).round()).abs() < 1e-8);
            assert!(center - width * 0.5 <= offset - 2.6 - 3.0 * texel);
            assert!(center + width * 0.5 >= offset + 2.6 + 3.0 * texel);
        }
    }

    #[test]
    fn offscreen_casters_keep_the_same_uv_and_valid_depth() {
        let mut input = input(2.0);
        input.light = Vec3::ONE.normalize();
        let fit = fit_shadow(input).unwrap();
        let receiver = fit.projection.project_point3(Vec3::ZERO);
        let caster_world = input.light * 12.0;
        assert!(input.view_projection.project_point3(caster_world).x > 1.0);
        let caster = fit.projection.project_point3(caster_world);
        assert!((caster.truncate() - receiver.truncate()).length() < 1e-6);
        assert!(caster.z < receiver.z);
        assert_covered(fit.projection, caster_world);
    }

    #[test]
    fn narrow_fits_cover_normal_offset_and_full_pcf_footprint() {
        for aspect in [0.001, 1.0, 1000.0] {
            let mut input = input(2.0);
            input.view_projection.x_axis.x /= aspect;
            let fit = fit_shadow(input).unwrap();
            for p in visible_vertices(input.bounds, input.view_projection) {
                for normal in [Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y] {
                    let biased = p.as_vec3() + normal * fit.normal_offset;
                    let ndc = fit.projection.project_point3(biased);
                    // A +/-1 tap plus bilinear filtering reaches up to 2 texels out.
                    let guard = 4.0 / SHADOW_SIZE as f32;
                    assert!(ndc.x.abs() + guard <= 1.0, "{aspect}: {ndc:?}");
                    assert!(ndc.y.abs() + guard <= 1.0, "{aspect}: {ndc:?}");
                }
            }
        }
    }

    #[test]
    fn covers_visible_points_for_orbits_aspects_and_projections() {
        for offset in [DVec3::ZERO, DVec3::new(32000.0, -32000.0, 32000.0)] {
            let bounds = Region::new(
                (offset - DVec3::splat(8.0)).as_ivec3().to_array(),
                (offset + DVec3::splat(8.0)).as_ivec3().to_array(),
            )
            .unwrap();
            for direction in [Vec3::ONE, Vec3::new(-2.0, 1.0, 3.0), Vec3::Y] {
                let eye = offset.as_vec3() + direction.normalize() * 24.0;
                let up = if direction == Vec3::Y {
                    Vec3::Z
                } else {
                    Vec3::Y
                };
                let view = Mat4::look_at_rh(eye, offset.as_vec3(), up);
                for aspect in [0.5, 1.0, 2.0] {
                    for projection in [
                        Mat4::orthographic_rh(
                            -4.0 * aspect,
                            4.0 * aspect,
                            -4.0,
                            4.0,
                            0.001,
                            4096.0,
                        ),
                        Mat4::perspective_rh(0.8, aspect, 0.001, 4096.0),
                    ] {
                        let input = ShadowInput {
                            bounds,
                            view_projection: projection * view,
                            light: Vec3::new(-0.3, 0.8, 0.5).normalize(),
                        };
                        let fit = fit_shadow(input).unwrap();
                        for x in -4..=4 {
                            for y in -4..=4 {
                                for z in -4..=4 {
                                    let p = offset.as_vec3()
                                        + Vec3::new(x as f32, y as f32, z as f32) * 2.0;
                                    let ndc = input.view_projection.project_point3(p);
                                    if ndc.x.abs() <= 1.0
                                        && ndc.y.abs() <= 1.0
                                        && (0.0..=1.0).contains(&ndc.z)
                                    {
                                        assert_covered(fit.projection, p);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn empty_and_touching_intersections_are_safe() {
        let mut input = input(2.0);
        input.bounds = Region::new([2, 2, 2], [4, 4, 4]).unwrap();
        assert!(fit_shadow(input).is_none());
        input.bounds = Region::new([1, 1, 1], [4, 4, 4]).unwrap();
        let fit = fit_shadow(input).unwrap();
        assert!(fit.projection.is_finite());
        assert_covered(fit.projection, Vec3::ONE);
    }

    #[test]
    fn cache_invalidates_on_view_light_bounds_or_mesh_changes_only() {
        let mut cache = ShadowCache::default();
        let mut input = input(8.0);
        assert!(cache.update(input, false));
        assert!(!cache.update(input, false));
        assert!(cache.update(input, true)); // Edits/deletions within unchanged bounds.
        assert!(!cache.update(input, false));
        input.light = Vec3::ONE.normalize();
        assert!(cache.update(input, false));
        input.view_projection.x_axis.x *= 2.0; // Zoom or aspect change.
        assert!(cache.update(input, false));
        input.bounds.max[0] += 1;
        assert!(cache.update(input, false));
        let visible = input;
        input.bounds = Region::new([100; 3], [101; 3]).unwrap();
        assert!(!cache.update(input, true));
        assert!(cache.fit.is_none());
        assert!(cache.update(visible, false));
    }
}
