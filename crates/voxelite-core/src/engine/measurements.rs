use super::*;
use std::{hint::black_box, time::Instant};

// Run without other CPU-heavy jobs:
// cargo test --release -p voxelite-core --features unbounded-voxels \
//     measure_execute_10m -- --ignored --nocapture
#[test]
#[ignore = "release-mode timing of 10M voxels; requires unbounded-voxels"]
fn measure_execute_10m() {
    let mut document = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    document.content.bounds = Region::new([0; 3], [250, 200, 200]).unwrap();
    for p in document.content.bounds.cells() {
        document.content.layers[0].voxels.set(p, 1);
    }
    assert_eq!(document.content.layers[0].voxels.len(), 10_000_000);
    let mut engine = Engine::new(document.clone()).unwrap();
    let mut measured = Engine::new(document).unwrap();
    let mut samples: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for i in 0..8 {
        let tx = Transaction {
            expected_revision: engine.revision(),
            id: Uuid::from_u128(10 + i),
            actor: Actor::new(Origin::Human),
            label: "Measure one voxel".into(),
            command: Command::SetVoxels {
                layer: Uuid::from_u128(2),
                voxels: vec![Voxel {
                    at: [124, 199, 100],
                    slot: (i % 2) as u16,
                }],
                mode: Mode::Place,
                mirror: Mirror::default(),
            },
        };
        let mut record = |name, start: Instant| {
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            if i > 0 {
                samples.entry(name).or_default().push(ms);
            }
        };
        let start = Instant::now();
        let receipt = engine.execute(tx.clone()).unwrap();
        record("execute", start);
        assert_eq!(receipt.changes.voxel_count(), 1);

        let start = Instant::now();
        let mut staged = (*measured.document).clone();
        record("clone", start);
        let mut work = crate::command::Work {
            used: 0,
            limit: measured.limits.work_units,
        };
        let start = Instant::now();
        crate::command::apply(
            &mut staged,
            &tx.command,
            tx.actor.origin,
            measured.head(),
            &mut work,
        )
        .unwrap();
        record("apply", start);
        let start = Instant::now();
        staged.validate_against(&measured.document).unwrap();
        record("validate", start);
        let start = Instant::now();
        let changes = compare(&measured.document.content, &staged.content);
        record("compare", start);
        let start = Instant::now();
        assert!(black_box(&staged) != black_box(measured.document.as_ref()));
        record("equality", start);
        let start = Instant::now();
        let bytes = black_box(patch_bytes(&measured.document, &staged));
        record("patch_bytes", start);
        let start = Instant::now();
        measured
            .install(staged, tx.id, tx.actor, tx.label, changes, bytes)
            .unwrap();
        record("install", start);
    }
    for (name, mut times) in samples {
        times.sort_by(f64::total_cmp);
        eprintln!(
            "{name}_ms: p50={:.3}, min={:.3}, max={:.3} (n={})",
            times[times.len() / 2],
            times[0],
            times[times.len() - 1],
            times.len()
        );
    }
}
