//! Deterministic terrain copied from the production render harness.
//! Generation is outside timed regions; no renderer dependency is introduced.
use crate::Result;
use serde::Deserialize;
use voxelite_core::*;
use voxelite_format::Project;

fn hash3(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA6B)
        ^ (z as u32).wrapping_mul(0xC2B2_AE35)
        ^ seed.wrapping_mul(0x27D4_EB2F);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^ (h >> 15)
}
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}
fn noise3(x: f64, y: f64, z: f64, seed: u32) -> f64 {
    let (xi, yi, zi) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let (fx, fy, fz) = (
        smooth(x - x.floor()),
        smooth(y - y.floor()),
        smooth(z - z.floor()),
    );
    let mut acc = 0.0;
    // Fixed accumulation order, no parallel reduction or fused multiply-add.
    for (dx, dy, dz) in [
        (0, 0, 0),
        (1, 0, 0),
        (0, 1, 0),
        (1, 1, 0),
        (0, 0, 1),
        (1, 0, 1),
        (0, 1, 1),
        (1, 1, 1),
    ] {
        let w = (if dx == 1 { fx } else { 1.0 - fx })
            * (if dy == 1 { fy } else { 1.0 - fy })
            * (if dz == 1 { fz } else { 1.0 - fz });
        acc += w * (f64::from(hash3(xi + dx, yi + dy, zi + dz, seed)) / f64::from(u32::MAX));
    }
    acc
}
fn fbm(x: f64, y: f64, z: f64, seed: u32) -> u32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
    for octave in 0..5 {
        sum += amp * noise3(x * freq, y * freq, z * freq, seed + octave);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    // Quantize once; all occupancy thresholds, ranks and height decisions are integers.
    (sum / norm * f64::from(1u32 << 24)).round() as u32
}
fn palette() -> Palette {
    Palette::from_colors(
        [
            (58, 122, 52),
            (74, 140, 60),
            (92, 156, 70),
            (108, 168, 82),
            (120, 86, 54),
            (140, 100, 62),
            (104, 74, 46),
            (112, 112, 118),
            (96, 98, 104),
            (128, 128, 134),
            (236, 240, 244),
            (214, 222, 230),
        ]
        .iter()
        .map(|&(r, g, b)| Swatch::rgb(r, g, b))
        .collect(),
    )
    .unwrap()
}
fn finish(cells: Vec<[i32; 4]>, bounds: Region, name: &str) -> Result<Project> {
    let voxels = Volume::deserialize(serde::de::value::SeqDeserializer::<
        _,
        serde::de::value::Error,
    >::new(cells.into_iter().map(|c| {
        serde::de::value::SeqDeserializer::<_, serde::de::value::Error>::new(c.into_iter())
    })))?;
    let mut doc = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    doc.content.name = name.into();
    doc.content.bounds = bounds;
    doc.content.palette = palette();
    doc.content.layers[0].voxels = voxels;
    Ok(Project::new(doc)?)
}
pub(crate) fn terrain(width: i32, max_height: i32, target: usize, name: &str) -> Result<Project> {
    let field: Vec<_> = (0..width * width)
        .map(|i| {
            fbm(
                f64::from(i % width) / 48.0,
                0.0,
                f64::from(i / width) / 48.0,
                7,
            )
        })
        .collect();
    let height =
        |scale: u64, f: u32| (1 + ((u64::from(f) * scale) >> 40)).min(max_height as u64) as i32;
    let (mut lo, mut hi) = (1u64 << 16, max_height as u64 * 5 * (1 << 15));
    while hi - lo > 1 {
        let mid = (hi + lo) / 2;
        let count: usize = field.iter().map(|&f| height(mid, f) as usize).sum();
        if count > target {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let half = width / 2;
    let mut cells = Vec::with_capacity(target);
    for (i, &f) in field.iter().enumerate() {
        let (x, z) = (i as i32 % width - half, i as i32 / width - half);
        let h = height(lo, f);
        for y in 0..h {
            let depth = h - 1 - y;
            let jitter = (hash3(x, y, z, 3) % 4) as i32;
            let slot = if depth == 0 {
                if h > max_height * 3 / 4 {
                    11 + jitter % 2
                } else {
                    1 + jitter
                }
            } else if depth < 4 {
                5 + jitter % 3
            } else {
                8 + (hash3(x, y, z, 5) % 3) as i32
            };
            cells.push([x, y, z, slot]);
        }
    }
    finish(
        cells,
        Region::new([-half, 0, -half], [half, max_height, half])?,
        name,
    )
}
