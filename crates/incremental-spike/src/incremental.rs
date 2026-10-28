//! Chunk inputs -> padded composite -> production mesh, plus independent plans.
use crate::{RenderError, RenderResult, production};
use salsa::Setter;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use voxelite_core::{Chunk, Palette, Pos, Snapshot};
use voxelite_render_plan::RenderPlan;

#[derive(Default)]
struct Counts {
    padded: AtomicUsize,
    meshes: AtomicUsize,
    plans: AtomicUsize,
}

#[salsa::db]
trait Db: salsa::Database {
    fn counts(&self) -> &Counts;
}

#[salsa::db]
#[derive(Clone, Default)]
struct Database {
    storage: salsa::Storage<Self>,
    counts: Arc<Counts>,
}

#[salsa::db]
impl salsa::Database for Database {}

#[salsa::db]
impl Db for Database {
    fn counts(&self) -> &Counts {
        &self.counts
    }
}

#[salsa::input]
struct PaletteInput {
    #[returns(ref)]
    value: Palette,
}

#[salsa::input]
struct ChunkInput {
    #[returns(copy)]
    at: Pos,
    #[returns(copy)]
    palette: PaletteInput,
    #[returns(ref)]
    neighbors: Vec<ChunkInput>,
    // Only chunks at this coordinate, ordered by visible layer precedence.
    #[returns(ref)]
    layers: Arc<Vec<Chunk>>,
}

#[salsa::input]
struct DocumentInput {
    #[returns(ref)]
    chunks: Vec<ChunkInput>,
    #[returns(copy)]
    exact_lighting: bool,
}

#[salsa::tracked(returns(ref))]
fn linear_palette(db: &dyn Db, input: PaletteInput) -> [Option<[f32; 3]>; 257] {
    let mut palette = [None; 257];
    for slot in &input.value(db).slots {
        palette[usize::from(slot.index)] = Some(std::array::from_fn(|a| {
            let v = f32::from(slot.color.rgba[a]) / 255.;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        }));
    }
    palette
}

#[salsa::tracked(returns(ref))]
fn padded_cells(db: &dyn Db, input: ChunkInput) -> Arc<[u16; 8000]> {
    db.counts().padded.fetch_add(1, Ordering::Relaxed);
    let at = input.at(db);
    let mut slots = [0; 8000];
    // The same two-cell composite and precedence as production LocalCells::new.
    // Equality here prevents an interior edit remeshing all 26 neighbor chunks.
    for neighbor in input.neighbors(db) {
        let offset: Pos = std::array::from_fn(|a| neighbor.at(db)[a] - at[a]);
        let lo = offset.map(|d| (-2 - d * 16).max(0));
        let hi = offset.map(|d| (18 - d * 16).min(16));
        for chunk in neighbor.layers(db).iter() {
            for z in lo[2]..hi[2] {
                for y in lo[1]..hi[1] {
                    for x in lo[0]..hi[0] {
                        let slot = chunk.get((x + 16 * y + 256 * z) as u16);
                        if slot != 0 {
                            let p: [usize; 3] = std::array::from_fn(|a| {
                                ([x, y, z][a] + offset[a] * 16 + 2) as usize
                            });
                            slots[(p[2] * 20 + p[1]) * 20 + p[0]] = slot;
                        }
                    }
                }
            }
        }
    }
    Arc::new(slots)
}

#[salsa::tracked(no_eq, returns(clone))]
fn chunk_mesh(db: &dyn Db, input: ChunkInput) -> Arc<production::ChunkMesh> {
    let cells = padded_cells(db, input);
    let palette = linear_palette(db, input.palette(db));
    db.counts().meshes.fetch_add(1, Ordering::Relaxed);
    Arc::new(production::from_padded(input.at(db), cells, palette))
}

#[salsa::tracked(returns(clone))]
fn render_plan(db: &dyn Db, document: DocumentInput) -> RenderPlan {
    db.counts().plans.fetch_add(1, Ordering::Relaxed);
    if document.exact_lighting(db) {
        RenderPlan::standard()
    } else {
        RenderPlan::default()
    }
}

type Geometry = Vec<(Pos, Arc<production::ChunkMesh>)>;

#[salsa::tracked(no_eq, returns(clone))]
fn document_meshes(db: &dyn Db, document: DocumentInput) -> RenderResult<Arc<Geometry>> {
    let mut quads = 0;
    let mut meshes = Vec::new();
    for &chunk in document.chunks(db) {
        let mesh = chunk_mesh(db, chunk);
        quads += mesh.vertices.len() / 6;
        if quads > voxelite_mesh::MAX_QUADS {
            return Err(RenderError::Limit("spike chunk mesh quad budget".into()));
        }
        if !mesh.vertices.is_empty() {
            meshes.push((chunk.at(db), mesh));
        }
    }
    Ok(Arc::new(meshes))
}

pub(crate) struct Pipeline {
    db: Database,
    chunks: BTreeMap<Pos, ChunkInput>,
    document: DocumentInput,
    palette: PaletteInput,
    previous: Snapshot,
    parallel: bool,
}

fn layers(snapshot: &Snapshot, at: Pos) -> Arc<Vec<Chunk>> {
    Arc::new(
        snapshot
            .layers
            .iter()
            .filter(|l| l.visible)
            .filter_map(|l| l.voxels.chunk(at).cloned())
            .collect(),
    )
}

fn occupied(snapshot: &Snapshot) -> BTreeSet<Pos> {
    snapshot
        .layers
        .iter()
        .filter(|l| l.visible)
        .flat_map(|l| l.voxels.chunk_positions())
        .collect()
}

impl Pipeline {
    pub(crate) fn new(snapshot: &Snapshot, exact: bool) -> Self {
        let mut db = Database::default();
        let palette = PaletteInput::builder(snapshot.palette.clone())
            .durability(salsa::Durability::HIGH)
            .new(&db);
        let lo = snapshot.bounds.min.map(|v| v.div_euclid(16));
        let hi = snapshot.bounds.max.map(|v| (v - 1).div_euclid(16));
        let mut chunks = BTreeMap::new();
        // Stable identities for empty as well as occupied coordinates. An added
        // voxel must invalidate reads of formerly absent neighbor chunks.
        for z in lo[2]..=hi[2] {
            for y in lo[1]..=hi[1] {
                for x in lo[0]..=hi[0] {
                    let at = [x, y, z];
                    chunks.insert(
                        at,
                        ChunkInput::builder(at, palette, Vec::new(), layers(snapshot, at))
                            .durability(salsa::Durability::HIGH)
                            .layers_durability(salsa::Durability::MEDIUM)
                            .new(&db),
                    );
                }
            }
        }
        for (&at, &input) in &chunks {
            let mut neighbors = Vec::new();
            for z in -1..=1 {
                for y in -1..=1 {
                    for x in -1..=1 {
                        if let Some(&neighbor) = chunks.get(&[at[0] + x, at[1] + y, at[2] + z]) {
                            neighbors.push(neighbor);
                        }
                    }
                }
            }
            input.set_neighbors(&mut db).to(neighbors);
        }
        let active = occupied(snapshot).iter().map(|at| chunks[at]).collect();
        // Low-durability view settings must not force validation of the entire
        // geometry graph. Chunk contents/membership are medium; palette is high.
        let document = DocumentInput::builder(active, exact)
            .chunks_durability(salsa::Durability::MEDIUM)
            .new(&db);
        Self {
            db,
            chunks,
            document,
            palette,
            previous: snapshot.clone(),
            parallel: true,
        }
    }

    pub(crate) fn sync(&mut self, snapshot: &Snapshot, exact: bool) {
        if self.previous.bounds != snapshot.bounds {
            *self = Self::new(snapshot, exact);
            return;
        }
        let full = !snapshot.layers.iter().map(|l| (l.id, l.visible)).eq(self
            .previous
            .layers
            .iter()
            .map(|l| (l.id, l.visible)));
        let changed: BTreeSet<_> = if full {
            self.chunks.keys().copied().collect()
        } else {
            snapshot
                .layers
                .iter()
                .zip(&self.previous.layers)
                .filter(|(l, _)| l.visible)
                .flat_map(|(l, old)| l.voxels.changed_chunks(&old.voxels))
                .collect()
        };
        self.parallel = changed.len() >= 4 || snapshot.palette != self.previous.palette;
        for &at in &changed {
            self.chunks[&at]
                .set_layers(&mut self.db)
                .to(layers(snapshot, at));
        }
        if !changed.is_empty() {
            let active: Vec<_> = occupied(snapshot)
                .iter()
                .map(|at| self.chunks[at])
                .collect();
            if active != *self.document.chunks(&self.db) {
                self.document.set_chunks(&mut self.db).to(active);
            }
        }
        if snapshot.palette != self.previous.palette {
            self.palette
                .set_value(&mut self.db)
                .to(snapshot.palette.clone());
        }
        if self.document.exact_lighting(&self.db) != exact {
            self.document.set_exact_lighting(&mut self.db).to(exact);
        }
        self.previous = snapshot.clone();
    }

    pub(crate) fn evaluate(&self) -> RenderResult<(Arc<Geometry>, RenderPlan)> {
        // Prewarm independent chunk queries on cloneable database handles. Clones
        // are dropped before any setters. Like production, cap scoped workers at 8.
        let chunks = self.document.chunks(&self.db);
        let workers = std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(8)
            .min(chunks.len());
        if self.parallel && workers > 1 {
            let next = AtomicUsize::new(0);
            std::thread::scope(|scope| {
                for _ in 0..workers {
                    let db = self.db.clone();
                    let next = &next;
                    scope.spawn(move || {
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(&chunk) = chunks.get(i) else { break };
                            std::hint::black_box(chunk_mesh(&db, chunk));
                        }
                    });
                }
            });
        }
        Ok((
            document_meshes(&self.db, self.document)?,
            render_plan(&self.db, self.document),
        ))
    }

    pub(crate) fn counts(&self) -> [usize; 3] {
        let c = &self.db.counts;
        [&c.padded, &c.meshes, &c.plans].map(|v| v.load(Ordering::Relaxed))
    }

    pub(crate) fn plan(&self) -> RenderPlan {
        render_plan(&self.db, self.document)
    }
}

#[cfg(test)]
mod tests;
