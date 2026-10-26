//! Reproducible CPU shadow timings and optional wide/close demo PNGs.
//! Run: cargo run --release -p voxelite-export --example shadow_bench -- /tmp/shadows
//! Uses 1024x640 because the public CPU renderer caps either dimension at 1024.
use std::{hint::black_box, path::Path, time::Instant};
use voxelite_core::*;
use voxelite_render::*;
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1);
    if let Some(path) = &output {
        std::fs::create_dir_all(path)?;
    }
    let project = voxelite_format::from_bytes(include_bytes!("autumn-workshop.voxl"))?;
    let snapshot = &project.document.content;
    let camera = Camera::fit(snapshot, PresetView::Iso, 1.6, false)?;
    for (name, camera) in [
        ("wide", camera),
        (
            "close",
            Camera {
                projection: Projection::Orthographic { height: 24.0 },
                ..camera
            },
        ),
    ] {
        let options = RenderOptions {
            width: 1024,
            height: 640,
            supersampling: 1,
            view: View::Camera(camera),
            background: [235, 235, 230, 255],
            ..Default::default()
        };
        let warmup = render(snapshot, &options)?;
        if let Some(path) = &output {
            let file = std::fs::File::create(Path::new(path).join(format!("{name}.png")))?;
            let mut encoder = png::Encoder::new(file, warmup.width, warmup.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&warmup.rgba)?;
        }
        let mut times = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            black_box(render(snapshot, &options)?);
            times.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        times.sort_by(f64::total_cmp);
        println!(
            "{name} CPU 1024x640, 1 sample: median {:.3} ms ({times:?})",
            times[times.len() / 2]
        );
    }
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
