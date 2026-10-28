//! Performance only: no texture captures, PNG operations or visual gates.
use crate::{
    Result, block_on,
    cli::Options,
    metrics::{Distribution, Metric, Spread},
    paths,
    report::{self, BenchReport, Load, Meta, Phase},
    scenes, style,
};
use quantum_core::*;
use quantum_render::{
    Camera,
    gpu::{FrameStats, Headless, HeadlessOptions},
};
use std::{collections::BTreeMap, time::Instant};

#[derive(Default)]
struct Samples {
    values: BTreeMap<String, Vec<f64>>,
    count: usize,
}
impl Samples {
    fn push(&mut self, stats: &FrameStats, edit: Option<(f64, f64)>) {
        self.count += 1;
        let mut add = |key: &str, v: f64| self.values.entry(key.into()).or_default().push(v);
        for (key, value) in [
            ("wall_ms", stats.wall_ms),
            ("submit_complete_ms", stats.submit_complete_ms),
            ("occupancy_sync_ms", stats.occupancy_sync_ms),
            ("mesh_sync_ms", stats.mesh_sync_ms),
            ("occupancy_upload_ms", stats.occupancy_upload_ms),
            ("mesh_upload_ms", stats.mesh_upload_ms),
            ("encode_ms", stats.encode_ms),
            ("draw_calls", f64::from(stats.draw_calls)),
            ("shadow_draw_calls", f64::from(stats.shadow_draw_calls)),
            ("triangles", stats.triangles as f64),
            ("vertices", stats.vertices as f64),
            ("upload_bytes", stats.upload_bytes as f64),
            ("chunks_updated", f64::from(stats.chunks_updated)),
            ("vertex_bytes", stats.vertex_bytes as f64),
            ("occupancy_bytes", stats.occupancy_bytes as f64),
            ("target_bytes", stats.target_bytes as f64),
        ] {
            add(key, value);
        }
        if let Some(bytes) = stats.device_allocated_bytes {
            add("device_allocated_bytes", bytes as f64);
        }
        if let Some(gpu) = stats.gpu {
            add("gpu_main_ms", gpu.main_ms);
            add("gpu_resolve_ms", gpu.resolve_ms);
            add("gpu_total_ms", gpu.total_ms);
            // Include zero for frames with no shadow redraw so sample counts stay aligned.
            add("gpu_shadow_ms", gpu.shadow_ms.unwrap_or(0.0));
        }
        if let Some((execute, clone)) = edit {
            add("execute_ms", execute);
            add("clone_ms", clone);
            add("edit_to_pixels_ms", execute + clone + stats.wall_ms);
        }
    }
    fn finish(mut self) -> Result<(usize, BTreeMap<String, Distribution>)> {
        // RSS measured once at the phase boundary, outside frame timing. It includes
        // engine history, CPU caches, driver allocations, and all process overhead.
        if let Some(rss) = report::rss_mb() {
            self.values.insert("rss_mb".into(), vec![rss]);
        }
        Ok((
            self.count,
            self.values
                .into_iter()
                .map(|(k, v)| Ok((k, Distribution::new(&v)?)))
                .collect::<Result<_>>()?,
        ))
    }
}
type RunPhases = BTreeMap<String, Vec<(usize, BTreeMap<String, Distribution>)>>;
fn record(out: &mut RunPhases, key: String, samples: Samples) -> Result<()> {
    let result = samples.finish()?;
    eprintln!(
        "{key}: wall p50 {:.3} ms, p95 {:.3} ms ({} samples)",
        result.1["wall_ms"].p50, result.1["wall_ms"].p95, result.0
    );
    out.entry(key).or_default().push(result);
    Ok(())
}
fn render(
    gpu: &mut Headless,
    snapshot: Option<Snapshot>,
    bounds: Region,
    camera: Camera,
    display: [u32; 2],
) -> Result<FrameStats> {
    Ok(block_on(gpu.render(snapshot, bounds, camera, display, style(bounds), false))?.stats)
}
pub fn run(options: &Options) -> Result<BenchReport> {
    let init = Instant::now();
    let mut gpu = block_on(Headless::new(HeadlessOptions {
        profile: true,
        ..Default::default()
    }))?;
    let init_ms = init.elapsed().as_secs_f64() * 1000.0;
    let mut meta = Meta::new(
        gpu.adapter_info(),
        options.display,
        options.frames,
        options.warmup,
        options.runs,
        init_ms,
    )?;
    let load_start = meta.load_average.start;
    let mut out = RunPhases::new();
    // Generate once; each run still gets fresh scene caches, targets, engines and phases.
    let loaded: Vec<_> = options
        .scenes
        .iter()
        .map(|name| {
            let scene = scenes::load(name)?;
            meta.scene_hashes
                .insert(name.clone(), scenes::content_hash(&scene)?);
            meta.scene_voxels.insert(
                name.clone(),
                scene.layers.iter().map(|l| l.voxels.len()).sum(),
            );
            let fit = paths::fit(&scene, options.display)?;
            Ok((name, scene, fit))
        })
        .collect::<Result<_>>()?;
    for run in 0..options.runs {
        let mut load = Load {
            start: report::load_average(),
            end: None,
        };
        for (name, snapshot, fit) in &loaded {
            eprintln!(
                "run {}/{}, scene {} ({} voxels, {})",
                run + 1,
                options.runs,
                name,
                meta.scene_voxels[*name],
                meta.scene_hashes[*name]
            );
            gpu.reset_scene()?;
            let bounds = snapshot.bounds;
            let cold = render(
                &mut gpu,
                Some(snapshot.clone()),
                bounds,
                *fit,
                options.display,
            )?;
            if cold.target_size != meta.surface_size {
                return Err("actual surface size differs from the production 2x contract".into());
            }
            let shadow = cold.shadow_path.as_str();
            if let Some(previous) = meta.shadow_path.insert((*name).clone(), shadow.into())
                && previous != shadow
            {
                return Err(format!("{name}: shadow path changed between runs ({previous} -> {shadow}); results are incomparable").into());
            }
            eprintln!(
                "{name}: shadow_path={shadow}, GPU timestamps={}",
                cold.gpu.is_some()
            );
            let mut samples = Samples::default();
            samples.push(&cold, None);
            record(&mut out, format!("{name}/cold"), samples)?;
            for phase in ["fit", "orbit_fit", "orbit_close"] {
                let mut samples = Samples::default();
                // Warm up the same path, then restart at angle zero for measured frames.
                for camera in paths::sequence(*fit, phase, options.warmup) {
                    render(&mut gpu, None, bounds, camera, options.display)?;
                }
                for camera in paths::sequence(*fit, phase, options.frames) {
                    let stats = render(&mut gpu, None, bounds, camera, options.display)?;
                    samples.push(&stats, None);
                }
                record(&mut out, format!("{name}/{phase}"), samples)?;
            }
            let mut engine = Engine::new(Document {
                id: Uuid::from_u128(1),
                content: snapshot.clone(),
                versions: Default::default(),
                lineage: None,
                extensions: Default::default(),
            })?;
            for (phase, apply, revert) in edits(snapshot, *name == "terrain10m")? {
                let mut samples = Samples::default();
                let iterations = if *name == "terrain10m" { 10 } else { 24 };
                for i in 0..iterations {
                    let command = if i % 2 == 0 {
                        apply.clone()
                    } else {
                        revert.clone()
                    };
                    let start = Instant::now();
                    let receipt = engine.execute(Transaction {
                        expected_revision: engine.revision(),
                        id: Uuid::new_v4(),
                        actor: Actor::new(Origin::Human),
                        label: phase.into(),
                        command,
                    })?;
                    let execute_ms = start.elapsed().as_secs_f64() * 1000.0;
                    if receipt.entry.is_none() {
                        return Err(format!(
                            "{name}/{phase}: edit was a no-op; refusing invalid measurement"
                        )
                        .into());
                    }
                    let start = Instant::now();
                    let snapshot = engine.document().content.clone();
                    let clone_ms = start.elapsed().as_secs_f64() * 1000.0;
                    let stats = render(&mut gpu, Some(snapshot), bounds, *fit, options.display)?;
                    if stats.shadow_path.as_str() != shadow {
                        return Err(format!("{name}: shadow path changed during edits").into());
                    }
                    if i >= 4 {
                        samples.push(&stats, Some((execute_ms, clone_ms)));
                    }
                }
                if engine.document().content != *snapshot {
                    return Err(
                        format!("{name}/{phase}: revert did not restore original content").into(),
                    );
                }
                record(&mut out, format!("{name}/{phase}"), samples)?;
            }
        }
        load.end = report::load_average();
        meta.run_load.push(load);
    }
    meta.load_average = Load {
        start: load_start,
        end: report::load_average(),
    };
    let results = out
        .into_iter()
        .map(|(key, runs)| {
            let keys: Vec<_> = runs[0].1.keys().cloned().collect();
            if runs.iter().any(|r| r.1.keys().ne(keys.iter())) {
                return Err(format!("{key}: metric availability changed between runs").into());
            }
            let metrics = keys
                .into_iter()
                .map(|k| {
                    let distributions = runs.iter().map(|r| r.1[&k].clone()).collect::<Vec<_>>();
                    Ok((k, Metric::from_runs(&distributions)?))
                })
                .collect::<Result<_>>()?;
            let fps_p50 = Spread::new(runs.iter().map(|r| 1000.0 / r.1["wall_ms"].p50).collect())?;
            Ok((
                key,
                Phase {
                    samples_per_run: runs.iter().map(|r| r.0).collect(),
                    fps_p50,
                    metrics,
                },
            ))
        })
        .collect::<Result<_>>()?;
    Ok(BenchReport {
        schema_version: 1,
        meta,
        results,
    })
}

/// Erase/restore avoids admission overflow at 1M and preserves every original cell.
/// Palette recolor restores the actual original swatch (including autumn's palette).
fn edits(snapshot: &Snapshot, large: bool) -> Result<Vec<(&'static str, Command, Command)>> {
    let layer = snapshot
        .layers
        .iter()
        .rev()
        .find(|l| l.visible && !l.locked && !l.voxels.is_empty())
        .ok_or("scene has no editable visible layer")?;
    let top = layer
        .voxels
        .iter()
        .min_by_key(|(p, _)| (p[0].abs() + p[2].abs(), -p[1], *p))
        .ok_or("empty edit layer")?;
    let mut edits = vec![];
    let set = |voxels| Command::SetVoxels {
        layer: layer.id,
        voxels,
        mode: Mode::Place,
        mirror: Mirror::default(),
    };
    edits.push((
        "edit_1voxel",
        set(vec![Voxel { at: top.0, slot: 0 }]),
        set(vec![Voxel {
            at: top.0,
            slot: top.1,
        }]),
    ));
    if !large {
        for (name, extent) in [
            ("edit_cube16", [16, 16, 16]),
            ("edit_cube40", [40, 40, 40]),
            ("edit_slab64", [64, 1, 64]),
        ] {
            let size: [i32; 3] = std::array::from_fn(|a| {
                extent[a].min(snapshot.bounds.max[a] - snapshot.bounds.min[a])
            });
            let min = std::array::from_fn(|a| {
                let ideal = top.0[a] - if a == 1 { size[a] - 1 } else { size[a] / 2 };
                ideal.clamp(snapshot.bounds.min[a], snapshot.bounds.max[a] - size[a])
            });
            let region = Region::new(min, std::array::from_fn(|a| min[a] + size[a]))?;
            let original = layer
                .voxels
                .sorted()
                .into_iter()
                .filter(|(p, _)| region.contains(*p))
                .map(|(at, slot)| Voxel { at, slot })
                .collect();
            edits.push((
                name,
                Command::Primitive {
                    layer: layer.id,
                    shape: Shape::Box {
                        region,
                        hollow: false,
                    },
                    mode: Mode::Erase,
                    slot: None,
                    mirror: Mirror::default(),
                },
                set(original),
            ));
        }
    }
    let old = snapshot
        .palette
        .get(top.1)
        .ok_or("edit slot absent")?
        .clone();
    let mut new = old.clone();
    new.rgba[0] ^= 0xff;
    new.rgba[1] ^= 0xff;
    edits.push((
        "edit_recolor",
        Command::Palette(PaletteEdit::Set {
            slot: top.1,
            color: new,
        }),
        Command::Palette(PaletteEdit::Set {
            slot: top.1,
            color: old,
        }),
    ));
    Ok(edits)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn autumn_edits_are_real_and_restore_content() {
        let scene = scenes::load("autumn").unwrap();
        let mut engine = Engine::new(Document {
            id: Uuid::from_u128(1),
            content: scene.clone(),
            versions: Default::default(),
            lineage: None,
            extensions: Default::default(),
        })
        .unwrap();
        for (phase, apply, revert) in edits(&scene, false).unwrap() {
            for command in [apply, revert] {
                let receipt = engine
                    .execute(Transaction {
                        expected_revision: engine.revision(),
                        id: Uuid::new_v4(),
                        actor: Actor::new(Origin::Human),
                        label: phase.into(),
                        command,
                    })
                    .unwrap();
                assert!(receipt.entry.is_some(), "{phase}");
            }
            assert_eq!(engine.document().content, scene, "{phase}");
        }
    }
}
