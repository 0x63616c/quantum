//! Manual timings for private occupancy operations, using the example fixture.
use crate::occupancy::{Occupancy, OccupancyCache};
use quantum_core::*;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "manual occupancy timing; run with --release --ignored --nocapture"]
fn occupancy_timings() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let project =
        quantum_format::from_bytes(include_bytes!("../../../../examples/autumn-workshop.voxl"))?;
    let snapshot = &project.document.content;
    let start = Instant::now();
    for _ in 0..200 {
        black_box(Occupancy::from_snapshot(snapshot)?);
    }
    println!(
        "occupancy full build: {:.3} us",
        start.elapsed().as_secs_f64() * 1e6 / 200.0
    );
    let mut occupancy = Occupancy::from_snapshot(snapshot)?;
    let (&at, _) = snapshot.composite().first_key_value().unwrap();
    occupancy.clear_dirty();
    let start = Instant::now();
    for i in 0..20000 {
        black_box(occupancy.set(at, i % 2 != 0));
        black_box(occupancy.dirty_ranges());
        occupancy.clear_dirty();
    }
    println!(
        "single cell + dirty ranges: {:.3} us",
        start.elapsed().as_secs_f64() * 1e6 / 20000.0
    );
    let mut engine = Engine::new(project.document.clone())?;
    let layer = snapshot.sample(at).unwrap().1;
    engine.execute(Transaction {
        expected_revision: engine.revision(),
        id: Uuid::new_v4(),
        actor: Actor::new(Origin::Human),
        label: "Benchmark erase".into(),
        command: Command::SetVoxels {
            layer,
            voxels: vec![Voxel { at, slot: 0 }],
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
    })?;
    let edited = &engine.document().content;
    let mut cache = OccupancyCache::default();
    cache.sync(snapshot)?;
    cache.grid_mut().unwrap().clear_dirty();
    let start = Instant::now();
    for i in 0..1000 {
        cache.sync(if i % 2 == 0 { edited } else { snapshot })?;
        black_box(cache.grid().unwrap().dirty_ranges());
        cache.grid_mut().unwrap().clear_dirty();
    }
    println!(
        "snapshot single-cell sync + dirty ranges: {:.3} us",
        start.elapsed().as_secs_f64() * 1e6 / 1000.0
    );
    println!("occupancy buffer: {} bytes", occupancy.words().len() * 4);
    Ok(())
}
