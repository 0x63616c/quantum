//! Fixed scenes. Generation is outside measured frame time.
use crate::Result;
use serde::Deserialize;
use voxelite_core::*;

pub const DEFAULT_SCENES: &[&str] = &["autumn", "terrain1m", "sponge1m"];
pub fn load(name: &str) -> Result<Snapshot> {
    let snapshot = match name {
        "autumn" => {
            voxelite_format::from_bytes(include_bytes!("../../../examples/autumn-workshop.voxl"))?
                .document
                .content
        }
        "terrain1m" => terrain(128, 128, 1_040_000, name)?,
        "sponge1m" => sponge(128, 1_040_000)?,
        #[cfg(feature = "large-scenes")]
        "terrain10m" => terrain(256, 256, 10_000_000, name)?,
        #[cfg(not(feature = "large-scenes"))]
        "terrain10m" => return Err(
            "terrain10m requires --features large-scenes (cargo xtask enables it automatically)"
                .into(),
        ),
        _ => {
            return Err(format!(
                "unknown scene {name:?}; use autumn, terrain1m, sponge1m or terrain10m"
            )
            .into());
        }
    };
    snapshot.validate()?;
    Ok(snapshot)
}

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
fn finish(cells: Vec<[i32; 4]>, bounds: Region, name: &str) -> Result<Snapshot> {
    let voxels = Volume::deserialize(serde::de::value::SeqDeserializer::<
        _,
        serde::de::value::Error,
    >::new(cells.into_iter().map(|c| c.to_vec())))?;
    let mut doc = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    doc.content.name = name.into();
    doc.content.bounds = bounds;
    doc.content.palette = palette();
    doc.content.layers[0].voxels = voxels;
    Ok(doc.content)
}
fn terrain(width: i32, max_height: i32, target: usize, name: &str) -> Result<Snapshot> {
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
fn sponge(side: i32, target: usize) -> Result<Snapshot> {
    let n = side as usize;
    let field: Vec<_> = (0..n * n * n)
        .map(|i| {
            fbm(
                (i % n) as f64 / 14.0,
                ((i / n) % n) as f64 / 14.0,
                (i / (n * n)) as f64 / 14.0,
                11,
            )
        })
        .collect();
    // Index breaks quantized ties, producing exactly target voxels deterministically.
    let mut ranked: Vec<_> = field
        .iter()
        .copied()
        .enumerate()
        .map(|(i, f)| (f, i))
        .collect();
    ranked.sort_unstable();
    let threshold = ranked[ranked.len() - target];
    let half = side / 2;
    let cells = field
        .iter()
        .enumerate()
        .filter(|&(i, &f)| (f, i) >= threshold)
        .map(|(i, _)| {
            let (x, y, z) = (
                (i % n) as i32 - half,
                ((i / n) % n) as i32,
                (i / (n * n)) as i32 - half,
            );
            [x, y, z, 1 + (hash3(x, y, z, 9) % 10) as i32]
        })
        .collect();
    finish(
        cells,
        Region::new([-half, 0, -half], [half, side, half])?,
        "sponge1m",
    )
}

/// Streaming FNV-1a of serde JSON. Volume serialization sorts cells; snapshot maps
/// are BTreeMaps. Includes bounds, palette and layer visibility/order, not just voxels.
pub fn content_hash(snapshot: &Snapshot) -> Result<String> {
    struct Hash(u64);
    impl std::io::Write for Hash {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            for b in bytes {
                self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x100000001b3);
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut hash = Hash(0xcbf29ce484222325);
    serde_json::to_writer(&mut hash, snapshot)?;
    Ok(format!("fnv1a64-v1:{:016x}", hash.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn autumn_hash_is_stable() {
        let a = load("autumn").unwrap();
        assert_eq!(content_hash(&a).unwrap(), "fnv1a64-v1:653a1baee5288283");
    }
    #[test]
    fn default_build_keeps_production_voxel_budget() {
        #[cfg(not(feature = "large-scenes"))]
        assert_eq!(MAX_VOXELS, 1_048_576);
        #[cfg(feature = "large-scenes")]
        assert_eq!(MAX_VOXELS, 16_777_216);
    }
    #[test]
    fn small_generators_repeat_exactly() {
        let a = terrain(8, 8, 200, "tiny").unwrap();
        let b = terrain(8, 8, 200, "tiny").unwrap();
        assert_eq!(a, b);
        assert!(a.layers[0].voxels.len() <= 200);
        assert_eq!(sponge(8, 200).unwrap(), sponge(8, 200).unwrap());
    }
}
