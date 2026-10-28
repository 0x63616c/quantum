//! Throwaway incremental-query benchmark. See README.md for scope and caveats.
#[cfg(feature = "salsa")]
mod incremental;
mod scene;
mod production {
    include!(concat!(env!("OUT_DIR"), "/production_mesh.rs"));
}

use std::{hint::black_box, time::Instant};
use voxelite_core::{Pos, Snapshot};
use voxelite_render_plan::RenderPlan;

type RenderError = voxelite_mesh::MeshError;
type RenderResult<T> = std::result::Result<T, RenderError>;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum Pipeline {
    Current(production::ChunkMeshes),
    #[cfg(feature = "salsa")]
    Salsa(Box<incremental::Pipeline>),
}

impl Pipeline {
    fn new(mode: &str, snapshot: &Snapshot, exact: bool) -> Result<Self> {
        match mode {
            "current" => Ok(Self::Current(production::ChunkMeshes::default())),
            #[cfg(feature = "salsa")]
            "salsa" => Ok(Self::Salsa(Box::new(incremental::Pipeline::new(
                snapshot, exact,
            )))),
            _ => {
                let _ = (snapshot, exact);
                Err(format!("unavailable mode {mode}").into())
            }
        }
    }

    fn sync(&mut self, snapshot: &Snapshot, exact: bool) -> Result<usize> {
        match self {
            Self::Current(cache) => {
                let updates = cache.sync_validated(snapshot)?;
                black_box(if exact {
                    RenderPlan::standard()
                } else {
                    RenderPlan::default()
                });
                Ok(updates.len())
            }
            #[cfg(feature = "salsa")]
            Self::Salsa(cache) => {
                cache.sync(snapshot, exact);
                let result = cache.evaluate()?;
                black_box(result);
                Ok(0)
            }
        }
    }

    fn cold(&mut self, snapshot: &Snapshot, exact: bool) -> Result<()> {
        match self {
            Self::Current(_) => {
                self.sync(snapshot, exact)?;
            }
            #[cfg(feature = "salsa")]
            Self::Salsa(cache) => {
                black_box(cache.evaluate()?);
            }
        }
        Ok(())
    }

    fn counts(&self) -> [usize; 3] {
        match self {
            Self::Current(_) => [0; 3],
            #[cfg(feature = "salsa")]
            Self::Salsa(cache) => cache.counts(),
        }
    }

    fn digest(&self) -> Result<(u64, usize, usize)> {
        match self {
            Self::Current(cache) => Ok(digest(cache.meshes.iter().map(|(at, mesh)| (*at, mesh)))),
            #[cfg(feature = "salsa")]
            Self::Salsa(cache) => {
                let (meshes, _) = cache.evaluate()?;
                Ok(digest(meshes.iter().map(|(at, mesh)| (*at, mesh.as_ref()))))
            }
        }
    }
}

fn digest<'a>(
    meshes: impl Iterator<Item = (Pos, &'a production::ChunkMesh)>,
) -> (u64, usize, usize) {
    let mut hash = 0xcbf29ce484222325u64;
    let mut feed = |v: u32| {
        for b in v.to_le_bytes() {
            hash = (hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    };
    let (mut vertices, mut faces) = (0, 0);
    for (at, mesh) in meshes {
        for v in at {
            feed(v as u32);
        }
        vertices += mesh.vertices.len();
        faces += mesh.exposed_faces;
        feed(mesh.exposed_faces as u32);
        for v in &mesh.vertices {
            for f in v.position.into_iter().chain(v.normal).chain(v.color_ao) {
                feed(f.to_bits());
            }
        }
    }
    (hash, vertices, faces)
}

fn memory() -> Result<serde_json::Value> {
    let status = std::fs::read_to_string("/proc/self/status")?;
    let value = |key: &str| -> Option<u64> {
        status
            .lines()
            .find(|l| l.starts_with(key))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    };
    Ok(
        serde_json::json!({"rss_kib": value("VmRSS:"), "peak_rss_kib": value("VmHWM:"), "anonymous_kib": value("RssAnon:")}),
    )
}

fn percentile(values: &[f64], fraction: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[((sorted.len() - 1) as f64 * fraction).round() as usize]
}

fn diagnose(before: &Snapshot, after: &Snapshot, runs: usize) -> serde_json::Value {
    let mut chunks = Vec::new();
    let mut cells = Vec::new();
    let (mut chunk_count, mut cell_count) = (0, 0);
    for _ in 0..runs {
        let start = Instant::now();
        chunk_count = black_box(
            after.layers[0]
                .voxels
                .changed_chunks(&before.layers[0].voxels),
        )
        .len();
        chunks.push(start.elapsed().as_secs_f64() * 1e6);
        let start = Instant::now();
        cell_count = black_box(
            after.layers[0]
                .voxels
                .changed_cells(&before.layers[0].voxels),
        )
        .len();
        cells.push(start.elapsed().as_secs_f64() * 1e6);
    }
    serde_json::json!({
        "changed_chunks": chunk_count, "changed_cells": cell_count,
        "chunk_diff_median_us": percentile(&chunks, 0.5), "cell_diff_median_us": percentile(&cells, 0.5),
        "chunk_diff_samples_us": chunks, "cell_diff_samples_us": cells,
    })
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        return Err("usage: incremental-spike <current|salsa|diff> <single|boundary|bulk|full|palette|noop|settings> [runs=9] [size=256]; 256 needs --features large-scenes".into());
    }
    let (mode, workload) = (&args[0], &args[1]);
    let runs: usize = args.get(2).map_or(Ok(9), |v| v.parse())?;
    let size: i32 = args.get(3).map_or(Ok(256), |v| v.parse())?;
    if runs < 3 || !(32..=256).contains(&size) || size % 16 != 0 {
        return Err("runs must be >=3; size must be a multiple of 16 in 32..=256".into());
    }
    let before = scene::terrain(size)?;
    let after = scene::variant(&before, workload)?;
    if mode == "diff" {
        println!("{}", diagnose(&before, &after, runs));
        return Ok(());
    }
    let input_memory = memory()?;
    let mut pipeline = Pipeline::new(mode, &before, true)?;
    pipeline.cold(&before, true)?;
    let warm_memory = memory()?;
    let mut elapsed = Vec::new();
    let mut padded = Vec::new();
    let mut meshes = Vec::new();
    let mut plans = Vec::new();
    let mut updates = Vec::new();
    for i in 0..runs + 2 {
        let snapshot = if i % 2 == 0 { &after } else { &before };
        let exact = workload != "settings" || i % 2 != 0;
        let counts = pipeline.counts();
        if workload == "full" {
            // Destruction of the previous cache is outside the cold-build timer.
            drop(pipeline);
            let start = Instant::now();
            pipeline = Pipeline::new(mode, snapshot, exact)?;
            pipeline.cold(snapshot, exact)?;
            let micros = start.elapsed().as_secs_f64() * 1e6;
            if i >= 2 {
                elapsed.push(micros);
            }
        } else {
            let start = Instant::now();
            let updated = pipeline.sync(snapshot, exact)?;
            let micros = start.elapsed().as_secs_f64() * 1e6;
            if i >= 2 {
                elapsed.push(micros);
                updates.push(updated);
            }
        }
        if i >= 2 {
            let current = pipeline.counts();
            let old = if workload == "full" { [0; 3] } else { counts };
            padded.push(current[0] - old[0]);
            meshes.push(current[1] - old[1]);
            plans.push(current[2] - old[2]);
        }
    }
    let final_memory = memory()?;
    let fingerprint = pipeline.digest()?;
    // Isolate the tiny pure plan operation too. This does NOT benchmark Host's
    // callbacks/quarantine/timings and does not imply those may be memoized.
    let start = Instant::now();
    for _ in 0..10_000 {
        match &pipeline {
            Pipeline::Current(_) => {
                black_box(RenderPlan::standard());
            }
            #[cfg(feature = "salsa")]
            Pipeline::Salsa(cache) => {
                black_box(cache.plan());
            }
        }
    }
    let plan_ns = start.elapsed().as_nanos() as f64 / 10_000.;
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "mode": mode, "workload": workload, "runs": runs, "size": size,
            "voxels": before.layers[0].voxels.len(), "chunks": before.layers[0].voxels.chunk_count(),
            "available_workers": std::thread::available_parallelism()?.get(),
            "median_us": percentile(&elapsed, 0.5), "min_us": percentile(&elapsed, 0.), "max_us": percentile(&elapsed, 1.),
            "samples_us": elapsed, "padded_executions": padded, "mesh_executions": meshes, "plan_executions": plans,
            "current_buffer_updates": updates, "input_memory": input_memory, "warm_memory": warm_memory, "final_memory": final_memory,
            "geometry_digest": format!("{:016x}", fingerprint.0), "vertices": fingerprint.1, "exposed_faces": fingerprint.2,
            "plan_call_mean_ns": plan_ns, "loadavg": std::fs::read_to_string("/proc/loadavg")?.trim(),
        }))?
    );
    Ok(())
}
