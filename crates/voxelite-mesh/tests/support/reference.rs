//! Frozen pre-merge snapshot mesher: an independent ordering and budget oracle.
use std::collections::{BTreeMap, HashMap};
use voxelite_core::{Pos, Snapshot};
use voxelite_mesh::{MAX_QUADS, Mesh, MeshError, MeshResult, Quad};

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
