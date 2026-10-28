//! Launcher-only artwork treatment; shares the production mesh rasterizer.
use super::*;
use std::collections::HashSet;

pub(super) fn frame(snapshot: &Snapshot, mut camera: Camera, options: &RenderOptions) -> Camera {
    let cells = snapshot.composite();
    if cells.is_empty() {
        return camera;
    }
    let eye = DVec3::from(camera.eye);
    let target = DVec3::from(camera.target);
    let forward = (target - eye).normalize();
    let right = forward.cross(DVec3::from(camera.up)).normalize();
    let up = right.cross(forward).normalize();
    let mut lo = DVec2::splat(f64::INFINITY);
    let mut hi = DVec2::splat(f64::NEG_INFINITY);
    for cell in cells.keys() {
        for corner in 0..8 {
            let p = DVec3::from(std::array::from_fn::<_, 3, _>(|a| {
                f64::from(cell[a]) + f64::from((corner >> a) & 1)
            })) - target;
            let p = DVec2::new(p.dot(right), p.dot(up));
            lo = lo.min(p);
            hi = hi.max(p);
        }
    }
    let center = (lo + hi) * 0.5;
    let offset = right * center.x + up * center.y;
    camera.target = (target + offset).to_array();
    camera.eye = (eye + offset).to_array();
    camera.projection = Projection::Orthographic {
        height: (hi.y - lo.y)
            .max((hi.x - lo.x) * f64::from(options.height) / f64::from(options.width))
            / 0.88,
    };
    camera
}

pub(super) struct Artwork {
    shades: HashMap<(Pos, u8), f64>,
    pub shadow: Vec<f64>,
}
impl Artwork {
    pub fn new(snapshot: &Snapshot, basis: &Basis, options: &RenderOptions) -> Self {
        let cells: HashSet<_> = snapshot.composite().into_keys().collect();
        let mut shades = HashMap::new();
        for &p in &cells {
            for axis in 0..3 {
                for positive in [false, true] {
                    let direction = if positive { 1 } else { -1 };
                    let mut neighbor = p;
                    neighbor[axis] += direction;
                    if cells.contains(&neighbor) {
                        continue;
                    }
                    let mut count = 0;
                    for i in -1..=1 {
                        for j in -1..=1 {
                            if i == 0 && j == 0 {
                                continue;
                            }
                            let mut q = neighbor;
                            q[(axis + 1) % 3] += i;
                            q[(axis + 2) % 3] += j;
                            count += usize::from(cells.contains(&q));
                        }
                    }
                    let top = axis == 1 && positive;
                    let column = if top {
                        (2..7)
                            .filter(|j| cells.contains(&[p[0], p[1] + j, p[2]]))
                            .count()
                    } else {
                        0
                    };
                    let ao = if options.ambient_occlusion {
                        1.0 - (count as f64 * if top { 0.045 } else { 0.05 } + column as f64 * 0.04)
                            * options.ambient_occlusion_strength.get()
                    } else {
                        1.0
                    };
                    let light = match axis {
                        1 => 1.0,
                        2 => 0.80,
                        _ => 0.58,
                    };
                    shades.insert(
                        (p, axis as u8 * 2 + u8::from(positive)),
                        light * ao.max(0.1),
                    );
                }
            }
        }
        let mut shadow = vec![0.0; basis.width * basis.height];
        if options.shadows
            && let Some(ground) = cells.iter().map(|p| p[1]).min()
        {
            let feet: HashSet<_> = cells.iter().map(|p| [p[0], p[2]]).collect();
            let scale = match basis.projection {
                Projection::Orthographic { height } => basis.height as f64 / height,
                _ => 1.0,
            };
            for [x, z] in feet {
                let points = [[x, z], [x + 1, z], [x + 1, z + 1], [x, z + 1]].map(|[x, z]| {
                    basis.screen(
                        basis
                            .vertex(DVec3::new(f64::from(x), f64::from(ground), f64::from(z)))
                            .view,
                    ) + DVec2::new(0.0, scale * 0.49)
                });
                footprint(&points, basis.width, basis.height, &mut shadow);
            }
            // Three separable box passes approximate the reference Gaussian and
            // cost O(image pixels), independent of the scene's footprint area.
            let radius = (scale * 1.3).round().max(1.0) as usize;
            for _ in 0..3 {
                blur(&mut shadow, basis.width, basis.height, radius);
            }
        }
        Self { shades, shadow }
    }
    pub fn illumination(&self, f: Fragment) -> f64 {
        let axis = usize::from(f.face / 2);
        let mut p = f.world;
        p[axis] -= if f.face % 2 == 1 { 0.001 } else { -0.001 };
        self.shades
            .get(&(p.map(|v| v.floor() as i32), f.face))
            .copied()
            .unwrap_or(1.0)
    }
}
fn footprint(points: &[DVec2; 4], width: usize, height: usize, out: &mut [f64]) {
    let lo = points
        .iter()
        .fold(DVec2::splat(f64::INFINITY), |a, p| a.min(*p))
        .floor()
        .max(DVec2::ZERO);
    let hi = points
        .iter()
        .fold(DVec2::splat(f64::NEG_INFINITY), |a, p| a.max(*p))
        .ceil()
        .min(DVec2::new(width as f64, height as f64));
    for y in lo.y as usize..hi.y as usize {
        for x in lo.x as usize..hi.x as usize {
            let p = DVec2::new(x as f64 + 0.5, y as f64 + 0.5);
            let sides = std::array::from_fn::<_, 4, _>(|i| {
                cross(points[(i + 1) % 4] - points[i], p - points[i])
            });
            if sides.iter().all(|v| *v >= 0.0) || sides.iter().all(|v| *v <= 0.0) {
                out[y * width + x] = 90.0 / 255.0;
            }
        }
    }
}
fn blur(data: &mut Vec<f64>, width: usize, height: usize, radius: usize) {
    let mut temp = vec![0.0; data.len()];
    for (w, h, vertical) in [(width, height, false), (height, width, true)] {
        let index = |x: usize, y: usize| {
            if vertical {
                x * width + y
            } else {
                y * width + x
            }
        };
        for y in 0..h {
            let mut sum: f64 = (0..radius.min(w)).map(|x| data[index(x, y)]).sum();
            for x in 0..w {
                if x + radius < w {
                    sum += data[index(x + radius, y)];
                }
                temp[index(x, y)] = sum / (2 * radius + 1) as f64;
                if x >= radius {
                    sum -= data[index(x - radius, y)];
                }
            }
        }
        std::mem::swap(data, &mut temp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxelite_core::*;

    #[test]
    fn artwork_has_reference_face_lighting_transparency_and_soft_contact_shadow() {
        let layer = Uuid::new_v4();
        let mut engine = Engine::new(Document::new(Uuid::new_v4(), layer)).unwrap();
        engine
            .execute(Transaction {
                expected_revision: 0,
                id: Uuid::new_v4(),
                actor: Actor::new(Origin::Human),
                label: "Cube".into(),
                command: Command::Batch(vec![
                    Command::Palette(PaletteEdit::Set {
                        slot: 1,
                        color: Swatch::rgb(200, 200, 200),
                    }),
                    Command::SetVoxels {
                        layer,
                        voxels: vec![Voxel {
                            at: [0, 0, 0],
                            slot: 1,
                        }],
                        mode: Mode::Place,
                        mirror: Mirror::default(),
                    },
                ]),
            })
            .unwrap();
        let options = RenderOptions {
            width: 64,
            height: 64,
            thumbnail: true,
            ..Default::default()
        };
        let image = render(&engine.document().content, &options).unwrap();
        assert_eq!(image, render(&engine.document().content, &options).unwrap());
        let pixels = image.rgba.as_chunks::<4>().0;
        for brightness in [200, 160, 116] {
            assert!(
                pixels
                    .iter()
                    .any(|p| p[3] == 255 && p[0].abs_diff(brightness) <= 1),
                "missing face {brightness}"
            );
        }
        assert!(pixels.iter().any(|p| p[3] == 0));
        assert!(pixels.iter().any(|p| p[0] == 0 && p[3] > 0 && p[3] < 90));
        assert!(pixels.iter().any(|p| p[0] > 0 && p[3] > 0 && p[3] < 255));
        let without_shadow = render(
            &engine.document().content,
            &RenderOptions {
                shadows: false,
                ..options
            },
        )
        .unwrap();
        assert!(
            !without_shadow
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .any(|p| p[0] == 0 && p[3] > 0)
        );
    }
}
