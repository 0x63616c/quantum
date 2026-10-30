//! CPU benchmarks of the production greedy mesher; fixtures are built before timing.

use divan::{Bencher, black_box, counter::ItemsCount};
use voxelite_core::{
    Actor, CHUNK_SIZE, Command, Document, Engine, Mirror, Mode, Origin, Region, Snapshot,
    Transaction, Uuid, Voxel,
};
use voxelite_mesh::greedy_mesh;

fn main() {
    divan::main();
}

#[expect(
    clippy::expect_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn snapshot(voxels: Vec<Voxel>) -> Snapshot {
    let layer = Uuid::from_u128(2);
    let mut document = Document::new(Uuid::from_u128(1), layer);
    document.content.bounds =
        Region::new([-64, 0, -64], [64, 32, 64]).expect("valid benchmark bounds");
    let mut engine = Engine::new(document).expect("benchmark document must be valid");
    // SetVoxels admits at most 65,536 explicit cells per transaction.
    for (batch, cells) in voxels.chunks(65_536).enumerate() {
        engine
            .execute(Transaction {
                expected_revision: engine.revision(),
                id: Uuid::from_u128(3 + batch as u128),
                actor: Actor::new(Origin::Human),
                label: "Mesher benchmark fixture".into(),
                command: Command::SetVoxels {
                    layer,
                    voxels: cells.to_vec(),
                    mode: Mode::Place,
                    mirror: Mirror::default(),
                },
            })
            .expect("benchmark voxels must satisfy document limits");
    }
    engine.document().content.clone()
}

#[expect(
    clippy::expect_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn measure(bencher: Bencher, voxels: Vec<Voxel>) {
    let count = voxels.len();
    let snapshot = snapshot(voxels);
    let mesh = greedy_mesh(&snapshot).expect("fixture must mesh within production limits");
    assert_eq!(mesh.occupied, count);
    drop(mesh);
    bencher.counter(ItemsCount::new(count)).bench_local(|| {
        drop(black_box(
            greedy_mesh(black_box(&snapshot)).expect("validated fixture must mesh"),
        ));
    });
}

#[derive(Clone, Copy, Debug)]
enum Pattern {
    Solid,
    Checkerboard,
    Sparse,
    HollowShell,
}

#[divan::bench(args = [Pattern::Solid, Pattern::Checkerboard, Pattern::Sparse, Pattern::HollowShell], sample_count = 20)]
fn chunk(bencher: Bencher, pattern: Pattern) {
    let mut voxels = Vec::new();
    let mut seed = 1_u32;
    for z in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let occupied = match pattern {
                    Pattern::Solid => true,
                    Pattern::Checkerboard => (x + y + z) % 2 == 0,
                    Pattern::Sparse => seed >> 28 == 0,
                    Pattern::HollowShell => {
                        [x, y, z].into_iter().any(|v| v == 0 || v == CHUNK_SIZE - 1)
                    }
                };
                if occupied {
                    voxels.push(Voxel {
                        at: [x, y, z],
                        slot: 1,
                    });
                }
            }
        }
    }
    measure(bencher, voxels);
}

// Continuous terrain crosses chunk seams in all axes, with stepped hills and
// shallow caves. The largest fixture stays below the production voxel budget.
#[divan::bench(args = [2, 4, 8], sample_count = 20)]
fn terrain(bencher: Bencher, chunks_per_side: i32) {
    let side = chunks_per_side * CHUNK_SIZE;
    let mut voxels = Vec::new();
    for z in 0..side {
        for x in 0..side {
            let height = 16 + (x / 4 + z / 4) % 17;
            for y in 0..height {
                if y > 3 && y < height - 3 && (x / 5 + y / 3 + z / 5) % 11 == 0 {
                    continue;
                }
                voxels.push(Voxel {
                    at: [x - side / 2, y, z - side / 2],
                    slot: 1,
                });
            }
        }
    }
    measure(bencher, voxels);
}
