use crate::*;
use glam::{DVec2, DVec3};
use std::collections::HashMap;
use voxelite_core::{Pos, Region, Snapshot};

/// Named automatic framing or a fully specified camera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    /// Automatically frame the selected direction, optionally in perspective.
    Preset {
        /// Viewing direction.
        direction: PresetView,
        /// Use a fitted 45-degree perspective camera.
        perspective: bool,
    },
    /// Use the supplied explicit camera.
    Camera(Camera),
}
/// Bounded image and shading parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderOptions {
    /// Output width, 1..=1024.
    pub width: u32,
    /// Output height, 1..=1024.
    pub height: u32,
    /// Samples along each dimension, 1..=4; total samples at most 4,194,304.
    pub supersampling: u8,
    /// Automatic view or explicit camera.
    pub view: View,
    /// Straight-alpha sRGB background. Default is transparent.
    pub background: [u8; 4],
    /// Subtle local voxel contact shading.
    pub ambient_occlusion: bool,
    /// Cast deterministic directional shadows through the visible grid.
    pub shadows: bool,
    /// Darken silhouettes and visible creases.
    pub outline: bool,
}
impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            supersampling: 2,
            view: View::Preset {
                direction: PresetView::Iso,
                perspective: false,
            },
            background: [0; 4],
            ambient_occlusion: true,
            shadows: true,
            outline: false,
        }
    }
}
/// RGBA8 output plus the exact camera used for reproducible feedback.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderedImage {
    /// Output width.
    pub width: u32,
    /// Output height.
    pub height: u32,
    /// Row-major, top-to-bottom straight-alpha sRGB RGBA8.
    pub rgba: Vec<u8>,
    /// Captured fitted or explicit camera.
    pub camera: Camera,
    /// Number of greedy surface rectangles submitted.
    pub quads: usize,
}
#[derive(Clone, Copy)]
struct Fragment {
    depth: f32,
    world: [f32; 3],
    slot: u16,
    face: u8,
}
impl Default for Fragment {
    fn default() -> Self {
        Self {
            depth: f32::INFINITY,
            world: [0.0; 3],
            slot: 0,
            face: 0,
        }
    }
}
#[derive(Clone, Copy)]
struct Vertex {
    world: DVec3,
    view: DVec3,
}
struct Basis {
    eye: DVec3,
    right: DVec3,
    up: DVec3,
    forward: DVec3,
    projection: Projection,
    width: usize,
    height: usize,
    near: f64,
    far: f64,
}
impl Basis {
    fn vertex(&self, p: DVec3) -> Vertex {
        let d = p - self.eye;
        Vertex {
            world: p,
            view: DVec3::new(d.dot(self.right), d.dot(self.up), d.dot(self.forward)),
        }
    }
    fn screen(&self, p: DVec3) -> DVec2 {
        let height = match self.projection {
            Projection::Orthographic { height } => height,
            Projection::Perspective { fov_y_degrees } => {
                2.0 * p.z * (fov_y_degrees.to_radians() * 0.5).tan()
            }
        };
        let scale = self.height as f64 / height;
        DVec2::new(
            self.width as f64 * 0.5 + p.x * scale,
            self.height as f64 * 0.5 - p.y * scale,
        )
    }
}
/// Render an immutable validated snapshot on the CPU, with no window or adapter.
/// Lighting is evaluated in linear RGB, then supersamples resolve with premultiplied
/// alpha and one sRGB conversion. Geometry and shadow traversal are deterministic.
pub fn render(snapshot: &Snapshot, options: &RenderOptions) -> RenderResult<RenderedImage> {
    let samples = u64::from(options.width)
        * u64::from(options.height)
        * u64::from(options.supersampling).pow(2);
    if options.width == 0
        || options.height == 0
        || options.width > 1024
        || options.height > 1024
        || !(1..=4).contains(&options.supersampling)
        || samples > 4_194_304
    {
        return Err(RenderError::Limit(
            "render requires 1..=1024 pixels/axis, 1..=4 AA, at most 4,194,304 samples".into(),
        ));
    }
    let camera = match options.view {
        View::Camera(c) => {
            c.validate()?;
            c
        }
        View::Preset {
            direction,
            perspective,
        } => Camera::fit(
            snapshot,
            direction,
            f64::from(options.width) / f64::from(options.height),
            perspective,
        )?,
    };
    let mesh = greedy_mesh(snapshot)?;
    let ss = usize::from(options.supersampling);
    let width = options.width as usize * ss;
    let height = options.height as usize * ss;
    let eye = DVec3::from(camera.eye);
    let forward = (DVec3::from(camera.target) - eye).normalize();
    let right = forward.cross(DVec3::from(camera.up)).normalize();
    let up = right.cross(forward).normalize();
    let far = crate::camera::corners(snapshot.bounds)
        .map(|p| (p - eye).dot(forward))
        .fold(0.0, f64::max)
        + 1.0;
    let basis = Basis {
        eye,
        right,
        up,
        forward,
        projection: camera.projection,
        width,
        height,
        near: 0.001,
        far,
    };
    let mut fragments = vec![Fragment::default(); width * height];
    // Facing cull is performed in world space before clipping and projection.
    for q in &mesh.quads {
        let normal = DVec3::from(q.normal().map(f64::from));
        let points = q.vertices();
        let center = points
            .iter()
            .map(|p| DVec3::from(p.map(f64::from)))
            .sum::<DVec3>()
            * 0.25;
        let toward = match camera.projection {
            Projection::Orthographic { .. } => -forward,
            Projection::Perspective { .. } => eye - center,
        };
        if normal.dot(toward) <= 0.0 {
            continue;
        }
        let face = q.axis * 2 + u8::from(q.positive);
        let poly: Vec<_> = points
            .into_iter()
            .map(|p| basis.vertex(DVec3::from(p.map(f64::from))))
            .collect();
        let poly = clip(&clip(&poly, basis.near, true), basis.far, false);
        for i in 1..poly.len().saturating_sub(1) {
            triangle(
                &basis,
                [poly[0], poly[i], poly[i + 1]],
                q.slot,
                face,
                &mut fragments,
            );
        }
    }
    let cells: HashMap<_, _> = snapshot.composite().into_iter().collect();
    let light = (-right * 0.55 + up * 0.7 - forward * 0.65 + DVec3::Y * 0.65).normalize();
    let palette: HashMap<_, _> = snapshot
        .palette
        .slots
        .iter()
        .map(|s| {
            (
                s.index,
                [
                    linear(s.color.rgba[0]),
                    linear(s.color.rgba[1]),
                    linear(s.color.rgba[2]),
                ],
            )
        })
        .collect();
    let background = options.background;
    let bg = [
        linear(background[0]),
        linear(background[1]),
        linear(background[2]),
    ];
    let bg_alpha = f64::from(background[3]) / 255.0;
    let mut rgba = vec![0; options.width as usize * options.height as usize * 4];
    for y in 0..options.height as usize {
        for x in 0..options.width as usize {
            let mut accum = [0.0; 4];
            for dy in 0..ss {
                for dx in 0..ss {
                    let sx = x * ss + dx;
                    let sy = y * ss + dy;
                    let f = fragments[sy * width + sx];
                    let (rgb, alpha) = if f.slot == 0 {
                        (bg, bg_alpha)
                    } else {
                        let mut n = [0.0; 3];
                        n[(f.face / 2) as usize] = if f.face % 2 == 1 { 1.0 } else { -1.0 };
                        let normal = DVec3::from(n);
                        let p = DVec3::from(f.world.map(f64::from));
                        let diffuse = normal.dot(light).max(0.0);
                        let shadow = options.shadows
                            && diffuse > 0.0
                            && occluded(p + normal * 0.01, light, &cells, snapshot.bounds);
                        let ao = if options.ambient_occlusion {
                            ambient(p, normal, &cells)
                        } else {
                            1.0
                        };
                        let mut illumination =
                            (0.44 + 0.56 * diffuse * if shadow { 0.2 } else { 1.0 }) * ao;
                        if options.outline && edge(&fragments, width, height, sx, sy) {
                            illumination *= 0.55;
                        }
                        let color = palette[&f.slot];
                        (
                            [
                                color[0] * illumination,
                                color[1] * illumination,
                                color[2] * illumination,
                            ],
                            1.0,
                        )
                    };
                    for c in 0..3 {
                        accum[c] += rgb[c] * alpha;
                    }
                    accum[3] += alpha;
                }
            }
            let offset = (y * options.width as usize + x) * 4;
            if accum[3] > 0.0 {
                for c in 0..3 {
                    rgba[offset + c] = srgb(accum[c] / accum[3]);
                }
            }
            rgba[offset + 3] = (accum[3] / (ss * ss) as f64 * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    Ok(RenderedImage {
        width: options.width,
        height: options.height,
        rgba,
        camera,
        quads: mesh.quads.len(),
    })
}
fn clip(poly: &[Vertex], plane: f64, near: bool) -> Vec<Vertex> {
    let mut out = Vec::new();
    if poly.is_empty() {
        return out;
    }
    let inside = |v: Vertex| {
        if near {
            v.view.z >= plane
        } else {
            v.view.z <= plane
        }
    };
    let mut previous = *poly.last().unwrap();
    for &current in poly {
        if inside(current) != inside(previous) {
            let t = (plane - previous.view.z) / (current.view.z - previous.view.z);
            out.push(Vertex {
                world: previous.world.lerp(current.world, t),
                view: previous.view.lerp(current.view, t),
            });
        }
        if inside(current) {
            out.push(current);
        }
        previous = current;
    }
    out
}
fn cross(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}
fn triangle(b: &Basis, vertices: [Vertex; 3], slot: u16, face: u8, fragments: &mut [Fragment]) {
    let p = vertices.map(|v| b.screen(v.view));
    if p.iter().any(|p| !p.is_finite()) {
        return;
    }
    let area = cross(p[1] - p[0], p[2] - p[0]);
    if area.abs() < 1e-12 {
        return;
    }
    let min_x = p
        .iter()
        .map(|v| v.x)
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0)
        .min(b.width as f64) as usize;
    let max_x = p
        .iter()
        .map(|v| v.x)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .max(0.0)
        .min(b.width as f64) as usize;
    let min_y = p
        .iter()
        .map(|v| v.y)
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0)
        .min(b.height as f64) as usize;
    let max_y = p
        .iter()
        .map(|v| v.y)
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .max(0.0)
        .min(b.height as f64) as usize;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let sample = DVec2::new(x as f64 + 0.5, y as f64 + 0.5);
            let mut w = [
                cross(p[1] - sample, p[2] - sample) / area,
                cross(p[2] - sample, p[0] - sample) / area,
                cross(p[0] - sample, p[1] - sample) / area,
            ];
            if w.iter().any(|w| *w < -1e-9) {
                continue;
            }
            if matches!(b.projection, Projection::Perspective { .. }) {
                for i in 0..3 {
                    w[i] /= vertices[i].view.z;
                }
                let total = w.iter().sum::<f64>();
                for weight in &mut w {
                    *weight /= total;
                }
            }
            let depth = (0..3).map(|i| w[i] * vertices[i].view.z).sum::<f64>() as f32;
            let current = &mut fragments[y * b.width + x];
            if depth >= current.depth {
                continue;
            }
            let world = (0..3).map(|i| vertices[i].world * w[i]).sum::<DVec3>();
            *current = Fragment {
                depth,
                world: world.to_array().map(|v| v as f32),
                slot,
                face,
            };
        }
    }
}
fn ambient(p: DVec3, n: DVec3, cells: &HashMap<Pos, u16>) -> f64 {
    let axis = if n.x != 0.0 {
        0
    } else if n.y != 0.0 {
        1
    } else {
        2
    };
    let u = (axis + 1) % 3;
    let v = (axis + 2) % 3;
    let mut occupied = 0.0;
    for [du, dv] in [
        [-1.0, 0.0],
        [1.0, 0.0],
        [0.0, -1.0],
        [0.0, 1.0],
        [-0.7, -0.7],
        [-0.7, 0.7],
        [0.7, -0.7],
        [0.7, 0.7],
    ] {
        let mut q = (p + n * 0.65).to_array();
        q[u] += du * 1.1;
        q[v] += dv * 1.1;
        if cells.contains_key(&q.map(|c| c.floor() as i32)) {
            occupied += 1.0;
        }
    }
    1.0 - occupied * 0.035
}
fn occluded(origin: DVec3, direction: DVec3, cells: &HashMap<Pos, u16>, bounds: Region) -> bool {
    let o = origin.to_array();
    let d = direction.to_array();
    let mut p = o.map(|v| v.floor() as i32);
    let step = d.map(|v| {
        if v > 0.0 {
            1
        } else if v < 0.0 {
            -1
        } else {
            0
        }
    });
    let delta = d.map(|v| {
        if v == 0.0 {
            f64::INFINITY
        } else {
            1.0 / v.abs()
        }
    });
    let mut next: [f64; 3] = std::array::from_fn(|a| {
        if d[a] > 0.0 {
            (f64::from(p[a] + 1) - o[a]) / d[a]
        } else if d[a] < 0.0 {
            (f64::from(p[a]) - o[a]) / d[a]
        } else {
            f64::INFINITY
        }
    });
    for _ in 0..768 {
        if !bounds.contains(p) {
            return false;
        }
        if cells.contains_key(&p) {
            return true;
        }
        let t = next.into_iter().fold(f64::INFINITY, f64::min);
        for a in 0..3 {
            if next[a] <= t + 1e-10 {
                p[a] += step[a];
                next[a] += delta[a];
            }
        }
    }
    false
}
fn edge(f: &[Fragment], w: usize, h: usize, x: usize, y: usize) -> bool {
    let here = f[y * w + x];
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
            return true;
        }
        let other = f[ny as usize * w + nx as usize];
        if other.slot == 0 || (other.face != here.face && (other.depth - here.depth).abs() > 0.05) {
            return true;
        }
    }
    false
}
fn linear(c: u8) -> f64 {
    let v = f64::from(c) / 255.0;
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn srgb(c: f64) -> u8 {
    let v = if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}
