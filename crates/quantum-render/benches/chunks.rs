//! CPU-only benchmark of production chunk synchronization, including AO.
use divan::{Bencher, black_box};
use quantum_core::{Document, Region, Snapshot};
pub use quantum_render::{RenderError, RenderResult};

// Exercise private production geometry without exposing a benchmark API.
#[allow(dead_code)]
#[path = "../src/chunk_mesh.rs"]
mod chunk_mesh;

fn main() {
    divan::main();
}

#[expect(clippy::expect_used, reason = "benchmark fixtures must be valid")]
fn scene(chunks: i32, dense: bool) -> Snapshot {
    let mut snapshot = Document::default().content;
    snapshot.bounds = Region::new([0; 3], [128; 3]).expect("valid bounds");
    let mut cells = Vec::new();
    for i in 0..chunks {
        let base = [(i % 8) * 16, (i / 64) * 16, ((i / 8) % 8) * 16];
        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    if dense || (x + 3 * y + 7 * z) % 31 == 0 {
                        cells.push([base[0] + x, base[1] + y, base[2] + z, 1]);
                    }
                }
            }
        }
    }
    snapshot.layers[0].voxels =
        serde_json::from_value(serde_json::json!(cells)).expect("valid volume");
    snapshot.validate().expect("valid snapshot");
    snapshot
}

#[expect(clippy::expect_used, reason = "validated benchmark scene must mesh")]
fn measure(bencher: Bencher, chunks: i32, dense: bool) {
    let snapshot = scene(chunks, dense);
    // Include cold cache synchronization but exclude fixture construction. Warm
    // the worker pool before sampling; editor workers persist between frames.
    chunk_mesh::ChunkMeshes::default()
        .sync_validated(&snapshot)
        .expect("warmup meshes");
    bencher.bench_local(|| {
        let mut cache = chunk_mesh::ChunkMeshes::default();
        black_box(cache.sync_validated(black_box(&snapshot)).expect("mesh"));
        black_box(cache);
    });
}

#[divan::bench(args = [1, 3, 4, 8, 32, 128], sample_count = 100)]
fn solid(bencher: Bencher, chunks: i32) {
    measure(bencher, chunks, true);
}

#[divan::bench(args = [1, 3, 4, 8, 32, 128], sample_count = 100)]
fn sparse(bencher: Bencher, chunks: i32) {
    measure(bencher, chunks, false);
}
