//! Deterministic greedy geometry of immutable voxel snapshots.
//!
//! Exposes [`Quad`], [`Mesh`], [`greedy_mesh`], [`greedy_mesh_with_limit`],
//! [`MAX_QUADS`], [`MeshError`] and [`MeshResult`]; no renderer or GPU is required.
use std::collections::{BTreeMap, HashMap};
use voxelite_core::{Pos, Snapshot};

/// Maximum admitted greedy quads per mesh.
pub const MAX_QUADS: usize = 1_500_000;
/// Geometry or image admission/validation failure.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MeshError {
    /// Invalid source document.
    #[error(transparent)]
    Model(#[from] voxelite_core::Error),
    /// Geometry/image budget exceeded.
    #[error("render limit exceeded: {0}")]
    Limit(String),
    /// Render surface dimensions exceed device or allocation limits; resizing may help.
    #[error("render limit exceeded: {0}")]
    SurfaceSize(String),
    /// Invalid camera or render settings.
    #[error("invalid render settings: {0}")]
    Invalid(String),
}
/// Result from geometry or rendering operations.
pub type MeshResult<T> = Result<T, MeshError>;
/// One outward-facing greedy rectangle, using exact integer world coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quad {
    /// Lower corner on the face plane.
    pub origin: Pos,
    /// Extents along `(axis+1)%3` and `(axis+2)%3`.
    pub size: [u16; 2],
    /// Stable palette slot.
    pub slot: u16,
    /// Normal axis, 0/1/2.
    pub axis: u8,
    /// True for the positive normal.
    pub positive: bool,
}
impl Quad {
    /// Four counterclockwise vertices as seen from outside the solid.
    pub fn vertices(self) -> [[f32; 3]; 4] {
        let a = usize::from(self.axis);
        let u = (a + 1) % 3;
        let v = (a + 2) % 3;
        let p = self.origin.map(|c| c as f32);
        let mut b = p;
        b[u] += f32::from(self.size[0]);
        let mut c = b;
        c[v] += f32::from(self.size[1]);
        let mut d = p;
        d[v] += f32::from(self.size[1]);
        if self.positive {
            [p, b, c, d]
        } else {
            [p, d, c, b]
        }
    }
    /// Outward unit normal.
    pub fn normal(self) -> [f32; 3] {
        let mut n = [0.0; 3];
        n[usize::from(self.axis)] = if self.positive { 1.0 } else { -1.0 };
        n
    }
    /// Area in unit voxel faces.
    pub fn area(self) -> usize {
        usize::from(self.size[0]) * usize::from(self.size[1])
    }
}
/// Compact deterministic greedy geometry of the visible composite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mesh {
    /// Quads in axis, slice, sign, row, column order.
    pub quads: Vec<Quad>,
    /// Number of visible occupied composite cells.
    pub occupied: usize,
    /// Exposed unit faces before greedy merging.
    pub exposed_faces: usize,
}
/// Mesh the highest visible cell at every coordinate. Internal faces are culled
/// across chunk/layer boundaries; only identical stable colors merge.
pub fn greedy_mesh(snapshot: &Snapshot) -> MeshResult<Mesh> {
    greedy_mesh_with_limit(snapshot, MAX_QUADS)
}
/// Mesh with an explicit cap, useful for host resource admission.
pub fn greedy_mesh_with_limit(snapshot: &Snapshot, max_quads: usize) -> MeshResult<Mesh> {
    snapshot.validate()?;
    if max_quads > MAX_QUADS {
        return Err(MeshError::Invalid("quad budget exceeds hard limit".into()));
    }
    let cells: HashMap<_, _> = snapshot.composite().into_iter().collect();
    let mut mesh = Mesh {
        quads: vec![],
        occupied: cells.len(),
        exposed_faces: 0,
    };
    let Some(first) = cells.keys().next().copied() else {
        return Ok(mesh);
    };
    // Per-axis slices avoid retaining an unbounded six-face staging map. At most
    // one 256² mask and one copy of occupied coordinates exist at a time.
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        let min_u = cells.keys().fold(first[u], |min, p| min.min(p[u]));
        let min_v = cells.keys().fold(first[v], |min, p| min.min(p[v]));
        let width = (cells.keys().fold(first[u], |max, p| max.max(p[u])) - min_u + 1) as usize;
        let height = (cells.keys().fold(first[v], |max, p| max.max(p[v])) - min_v + 1) as usize;
        let mut slices: BTreeMap<i32, Vec<(Pos, u16)>> = BTreeMap::new();
        for (&p, &slot) in &cells {
            slices.entry(p[axis]).or_default().push((p, slot));
        }
        let mut mask = vec![0u16; width * height];
        for (slice, voxels) in slices {
            for positive in [false, true] {
                mask.fill(0);
                for (p, slot) in &voxels {
                    let mut neighbor = *p;
                    neighbor[axis] += if positive { 1 } else { -1 };
                    if !cells.contains_key(&neighbor) {
                        mask[(p[v] - min_v) as usize * width + (p[u] - min_u) as usize] = *slot;
                        mesh.exposed_faces += 1;
                    }
                }
                for y in 0..height {
                    for x in 0..width {
                        let slot = mask[y * width + x];
                        if slot == 0 {
                            continue;
                        }
                        let mut w = 1;
                        while x + w < width && mask[y * width + x + w] == slot {
                            w += 1;
                        }
                        let mut h = 1;
                        while y + h < height
                            && (0..w).all(|dx| mask[(y + h) * width + x + dx] == slot)
                        {
                            h += 1;
                        }
                        for dy in 0..h {
                            mask[(y + dy) * width + x..(y + dy) * width + x + w].fill(0);
                        }
                        if mesh.quads.len() == max_quads {
                            return Err(MeshError::Limit(format!(
                                "geometry exceeds {max_quads} greedy quads"
                            )));
                        }
                        let mut origin = [0; 3];
                        origin[axis] = slice + i32::from(positive);
                        origin[u] = min_u + x as i32;
                        origin[v] = min_v + y as i32;
                        mesh.quads.push(Quad {
                            origin,
                            size: [w as u16, h as u16],
                            slot,
                            axis: axis as u8,
                            positive,
                        });
                    }
                }
            }
        }
    }
    Ok(mesh)
}
