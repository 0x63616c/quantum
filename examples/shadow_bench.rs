//! Reproducible CPU shadow timings and optional wide/close demo PNGs.
//! Run: cargo run --release -p quantum-plugin-exporters --example shadow_bench -- /tmp/shadows
//! Occupancy internals: cargo test -p quantum-render --release --locked occupancy_timings -- --ignored --nocapture
//! Uses 1024x640 because the public CPU renderer caps either dimension at 1024.
use quantum_render::*;
use std::{hint::black_box, path::Path, time::Instant};
#[expect(clippy::print_stdout, reason = "CLI output")]
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1);
    if let Some(path) = &output {
        std::fs::create_dir_all(path)?;
    }
    let project = quantum_format::from_bytes(include_bytes!("autumn-workshop.voxl"))?;
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
    Ok(())
}
