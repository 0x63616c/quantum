//! Persistent composite mesh. Edits invalidate their 16³ chunks and AO neighbors.
use crate::{RenderError, RenderResult};
use quantum_core::{Pos, Snapshot};
use quantum_mesh::{MAX_QUADS, Mesh, Quad};
use std::collections::{BTreeMap, BTreeSet};

/// Vertex layout shared by the GPU backends. Color is linear RGB, AO is baked.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "gpu", derive(bytemuck::Pod, bytemuck::Zeroable))]
pub(crate) struct MeshVertex {
    /// World position.
    pub(crate) position: [f32; 3],
    /// Outward unit normal.
    pub(crate) normal: [f32; 3],
    /// Linear RGB followed by ambient illumination.
    pub(crate) color_ao: [f32; 4],
}
/// One chunk's greedy mesh, with AO discontinuities retained.
#[derive(Default, Debug)]
pub(crate) struct ChunkMesh {
    /// Triangle list, six vertices per greedy quad.
    pub(crate) vertices: Vec<MeshVertex>,
    /// Exposed unit faces, before merging.
    pub(crate) exposed_faces: usize,
}
/// A cache owned by one viewport. Camera motion never calls `sync_validated`.
#[derive(Default)]
pub(crate) struct ChunkMeshes {
    source: Option<Snapshot>,
    /// Deterministically ordered chunk geometry; empty chunks are absent.
    pub(crate) meshes: BTreeMap<Pos, ChunkMesh>,
}
impl ChunkMeshes {
    /// Synchronize a snapshot already validated by the document engine. Returns
    /// precisely the chunk buffers to replace/delete, including AO halo changes.
    /// Palette/layer-order/visibility changes invalidate the visible composite.
    #[cfg_attr(
        feature = "profiling",
        tracing::instrument(name = "mesh.sync", skip_all)
    )]
    pub(crate) fn sync_validated(&mut self, snapshot: &Snapshot) -> RenderResult<Vec<Pos>> {
        let full = self.source.as_ref().is_none_or(|old| {
            old.palette != snapshot.palette
                || old.bounds != snapshot.bounds
                || !old
                    .layers
                    .iter()
                    .map(|l| (l.id, l.visible))
                    .eq(snapshot.layers.iter().map(|l| (l.id, l.visible)))
        });
        let mut dirty = BTreeSet::new();
        if full {
            dirty.extend(self.meshes.keys().copied());
            for layer in snapshot.layers.iter().filter(|l| l.visible) {
                dirty.extend(layer.voxels.chunk_positions());
            }
        } else if let Some(old) = &self.source {
            let mut changed = BTreeSet::new();
            for (layer, previous) in snapshot
                .layers
                .iter()
                .zip(&old.layers)
                .filter(|(l, _)| l.visible)
            {
                changed.extend(
                    layer
                        .voxels
                        .changed_cells(&previous.voxels)
                        .into_iter()
                        .map(|(p, _, _)| p),
                );
            }
            for p in changed {
                let new = snapshot.sample(p).map(|(slot, _)| slot);
                if old.sample(p).map(|(slot, _)| slot) == new {
                    continue;
                }
                // Cull and AO dependencies extend two cells across chunk borders,
                // including diagonals. Interior edits only invalidate one chunk.
                let lo = p.map(|v| (v - 2).div_euclid(16));
                let hi = p.map(|v| (v + 2).div_euclid(16));
                for z in lo[2]..=hi[2] {
                    for y in lo[1]..=hi[1] {
                        for x in lo[0]..=hi[0] {
                            dirty.insert([x, y, z]);
                        }
                    }
                }
            }
        }
        let mut palette = [None; 257];
        for s in &snapshot.palette.slots {
            palette[usize::from(s.index)] = Some(std::array::from_fn(|a| {
                let v = f32::from(s.color.rgba[a]) / 255.;
                if v <= 0.04045 {
                    v / 12.92
                } else {
                    ((v + 0.055) / 1.055).powf(2.4)
                }
            }));
        }
        let dirty: Vec<_> = dirty.into_iter().collect();
        let meshes = mesh_dirty(&dirty, snapshot, &palette);
        let mut updates = Vec::new();
        let mut quads: usize = self.meshes.values().map(|m| m.vertices.len() / 6).sum();
        // Apply in dirty-set order: even a transient budget overflow must fail
        // exactly where the serial mesher did, before any later chunk removals.
        for (at, mesh) in dirty.into_iter().zip(meshes) {
            let mesh = match mesh {
                Ok(mesh) => mesh,
                Err(error) => {
                    self.source = None;
                    self.meshes.clear();
                    return Err(error);
                }
            };
            quads = quads - self.meshes.get(&at).map_or(0, |m| m.vertices.len() / 6)
                + mesh.vertices.len() / 6;
            if quads > MAX_QUADS {
                self.source = None;
                self.meshes.clear();
                return Err(RenderError::Limit("GPU chunk mesh quad budget".into()));
            }
            if !mesh.vertices.is_empty() {
                self.meshes.insert(at, mesh);
                updates.push(at);
            } else if self.meshes.remove(&at).is_some() {
                updates.push(at);
            }
        }
        self.source = Some(snapshot.clone());
        Ok(updates)
    }
}

// A unit face has vertices in 0..=16 relative to its chunk. Culling reads
// -1..=16. AO uses floor(vertex +/- 0.001 + 0.65*normal + 1.1*(du,dv)):
// the normal coordinate reaches -1..=16, and tangential coordinates -2..=17.
// Thus a two-cell halo suffices, including diagonals. Keep AO arithmetic in
// world coordinates to preserve f32 rounding at large/negative coordinates.
const HALO: i32 = 2;
const PADDED_SIZE: usize = 20;
struct LocalCells {
    min: Pos,
    slots: [u16; PADDED_SIZE * PADDED_SIZE * PADDED_SIZE],
}
impl LocalCells {
    fn new(at: Pos, snapshot: &Snapshot) -> Self {
        let mut cells = Self {
            min: at.map(|a| a * 16 - HALO),
            slots: [0; PADDED_SIZE * PADDED_SIZE * PADDED_SIZE],
        };
        // Later visible layers replace only occupied slots, just like sample().
        for layer in snapshot.layers.iter().filter(|l| l.visible) {
            for dz in -1..=1 {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let offset = [dx, dy, dz];
                        let neighbor = std::array::from_fn(|a| at[a] + offset[a]);
                        let Some(chunk) = layer.voxels.chunk(neighbor) else {
                            continue;
                        };
                        // Copy only the intersection with the padded block.
                        let lo = offset.map(|d| (-HALO - d * 16).max(0));
                        let hi = offset.map(|d| (16 + HALO - d * 16).min(16));
                        for z in lo[2]..hi[2] {
                            for y in lo[1]..hi[1] {
                                for x in lo[0]..hi[0] {
                                    let slot = chunk.get((x + 16 * y + 256 * z) as u16);
                                    if slot != 0 {
                                        let p = [x + dx * 16, y + dy * 16, z + dz * 16]
                                            .map(|v| (v + HALO) as usize);
                                        cells.slots
                                            [(p[2] * PADDED_SIZE + p[1]) * PADDED_SIZE + p[0]] =
                                            slot;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        cells
    }

    fn get(&self, p: Pos) -> u16 {
        let p: [usize; 3] = std::array::from_fn(|a| (p[a] - self.min[a]) as usize);
        debug_assert!(p.iter().all(|&v| v < PADDED_SIZE));
        self.slots[(p[2] * PADDED_SIZE + p[1]) * PADDED_SIZE + p[0]]
    }
}

fn mesh_dirty(
    dirty: &[Pos],
    snapshot: &Snapshot,
    palette: &[Option<[f32; 3]>],
) -> Vec<RenderResult<ChunkMesh>> {
    // Preserve the small-edit serial path and the existing eight-worker bound.
    // Reuse workers across syncs rather than allocating thread stacks each time.
    let pool = (dirty.len() >= 4).then(mesh_pool).flatten();
    mesh_dirty_with_pool(dirty, snapshot, palette, pool)
}

fn mesh_dirty_with_pool(
    dirty: &[Pos],
    snapshot: &Snapshot,
    palette: &[Option<[f32; 3]>],
    pool: Option<&rayon::ThreadPool>,
) -> Vec<RenderResult<ChunkMesh>> {
    let mesh = |&at: &Pos| mesh_chunk(at, &LocalCells::new(at, snapshot), palette);
    if let Some(pool) = pool {
        use rayon::prelude::*;
        // Indexed collection retains dirty-set order, including errors. Applying
        // results (and enforcing the aggregate quad budget) remains serial.
        return pool.install(|| dirty.par_iter().map(mesh).collect());
    }
    dirty.iter().map(mesh).collect()
}

fn mesh_pool() -> Option<&'static rayon::ThreadPool> {
    use std::sync::OnceLock;
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        let workers = std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(8);
        if workers <= 1 {
            return None;
        }
        // Thread allocation can fail under resource pressure. Serial meshing
        // remains correct; avoid a global-pool initialization panic in that case.
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .thread_name(|i| format!("quantum-mesh-{i}"))
            .build()
            .ok()
    })
    .as_ref()
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Face {
    slot: u16,
    ao: [u8; 4],
}
fn ambient(
    vertex: [f32; 3],
    center: [f32; 3],
    normal: [f32; 3],
    axis: usize,
    cells: &LocalCells,
) -> u8 {
    // Sample just inside this face at corners so adjacent convex faces do not
    // accidentally occlude themselves at exact integer boundaries.
    let p = std::array::from_fn::<_, 3, _>(|a| {
        vertex[a] + (center[a] - vertex[a]).signum() * 0.001 + normal[a] * 0.65
    });
    let mut occupied = 0;
    for [du, dv] in [
        [-1., 0.],
        [1., 0.],
        [0., -1.],
        [0., 1.],
        [-0.7, -0.7],
        [-0.7, 0.7],
        [0.7, -0.7],
        [0.7, 0.7],
    ] {
        let mut q = p;
        q[(axis + 1) % 3] += du * 1.1;
        q[(axis + 2) % 3] += dv * 1.1;
        occupied += u8::from(cells.get(q.map(|v| v.floor() as i32)) != 0);
    }
    occupied
}
fn face_mask(
    at: Pos,
    cells: &LocalCells,
    axis: usize,
    slice: i32,
    positive: bool,
) -> ([Face; 256], usize) {
    let u = (axis + 1) % 3;
    let v = (axis + 2) % 3;
    let mut exposed_faces = 0;
    let mut mask = [Face::default(); 256];
    for y in 0..16 {
        for x in 0..16 {
            let mut p = at.map(|a| a * 16);
            p[axis] += slice;
            p[u] += x;
            p[v] += y;
            let slot = cells.get(p);
            if slot == 0 {
                continue;
            }
            let mut neighbor = p;
            neighbor[axis] += if positive { 1 } else { -1 };
            if cells.get(neighbor) != 0 {
                continue;
            }
            let mut origin = p;
            origin[axis] += i32::from(positive);
            let quad = Quad {
                origin,
                size: [1, 1],
                slot,
                axis: axis as u8,
                positive,
            };
            let center =
                std::array::from_fn(|a| origin[a] as f32 + if a == axis { 0. } else { 0.5 });
            mask[(y * 16 + x) as usize] = Face {
                slot,
                ao: quad
                    .vertices()
                    .map(|p| ambient(p, center, quad.normal(), axis, cells)),
            };
            exposed_faces += 1;
        }
    }
    (mask, exposed_faces)
}

#[expect(
    clippy::expect_used,
    reason = "internal invariant: validated snapshot contains every meshed palette slot"
)]
fn emit_quad(mesh: &mut ChunkMesh, quad: Quad, face: Face, palette: &[Option<[f32; 3]>]) {
    let color = palette[usize::from(face.slot)]
        .as_ref()
        .expect("validated snapshot contains every meshed palette slot");
    let vertices = quad.vertices();
    // Pick the less conspicuous diagonal through differing AO values.
    let indices = if face.ao[0] as u16 + face.ao[2] as u16 > face.ao[1] as u16 + face.ao[3] as u16 {
        [0, 1, 3, 1, 2, 3]
    } else {
        [0, 1, 2, 0, 2, 3]
    };
    for i in indices {
        mesh.vertices.push(MeshVertex {
            position: vertices[i],
            normal: quad.normal(),
            color_ao: [
                color[0],
                color[1],
                color[2],
                1. - f32::from(face.ao[i]) * 0.035,
            ],
        });
    }
}

#[cfg_attr(
    feature = "profiling",
    tracing::instrument(name = "mesh.chunk", skip_all)
)]
fn mesh_chunk(
    at: Pos,
    cells: &LocalCells,
    palette: &[Option<[f32; 3]>],
) -> RenderResult<ChunkMesh> {
    let mut mesh = ChunkMesh::default();
    for axis in 0..3 {
        let u = (axis + 1) % 3;
        let v = (axis + 2) % 3;
        for slice in 0..16 {
            for positive in [false, true] {
                let (mut mask, exposed_faces) = face_mask(at, cells, axis, slice, positive);
                mesh.exposed_faces += exposed_faces;
                Mesh::greedy_rectangles(
                    &mut mask,
                    [16, 16],
                    Face::default(),
                    |face| face.ao.iter().all(|a| *a == face.ao[0]),
                    |[x, y], size, face| {
                        let mut origin = at.map(|a| a * 16);
                        origin[axis] += slice + i32::from(positive);
                        origin[u] += i32::from(x);
                        origin[v] += i32::from(y);
                        let quad = Quad {
                            origin,
                            size,
                            slot: face.slot,
                            axis: axis as u8,
                            positive,
                        };
                        emit_quad(&mut mesh, quad, face, palette);
                        Ok(())
                    },
                )?;
            }
        }
    }
    Ok(mesh)
}

#[cfg(test)]
#[path = "chunk_mesh/reference.rs"]
mod reference;
#[cfg(test)]
#[path = "chunk_mesh/tests.rs"]
mod tests;
