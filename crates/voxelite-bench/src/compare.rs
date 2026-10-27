//! Performance gates and markdown presentation; never reads images.
use crate::{
    Result,
    cli::Options,
    report::{BenchReport, Phase},
};
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
pub struct Tolerances {
    pub wall_pct: f64,
    pub wall_ms: f64,
    pub gpu_pct: f64,
    pub memory_pct: f64,
}
impl From<&Options> for Tolerances {
    fn from(o: &Options) -> Self {
        Self {
            wall_pct: o.wall_pct,
            wall_ms: o.wall_ms,
            gpu_pct: o.gpu_pct,
            memory_pct: o.memory_pct,
        }
    }
}
impl Default for Tolerances {
    fn default() -> Self {
        Self {
            wall_pct: 10.0,
            wall_ms: 0.3,
            gpu_pct: 10.0,
            memory_pct: 10.0,
        }
    }
}
pub fn regresses(before: f64, after: f64, pct: f64, absolute: f64) -> bool {
    after > before + absolute && after > before * (1.0 + pct / 100.0)
}
fn value(phase: &Phase, key: &str) -> Option<f64> {
    phase.metrics.get(key).map(|m| m.p50.median)
}
fn compatible(old: &BenchReport, new: &BenchReport) -> Result<()> {
    if old.schema_version != 1 || new.schema_version != 1 {
        return Err("unsupported bench schema; regenerate baseline with this harness".into());
    }
    if old.meta.machine_key != new.meta.machine_key {
        return Err(format!(
            "machine_key mismatch: {} vs {}; use a baseline recorded on {}",
            old.meta.machine_key, new.meta.machine_key, new.meta.machine_key
        )
        .into());
    }
    if old.meta.display_size != new.meta.display_size
        || old.meta.surface_size != new.meta.surface_size
        || old.meta.msaa != new.meta.msaa
        || old.meta.supersample != new.meta.supersample
        || old.meta.style != new.meta.style
        || old.meta.paths_version != new.meta.paths_version
        || old.meta.frames != new.meta.frames
        || old.meta.warmup != new.meta.warmup
    {
        return Err("render settings or camera sampling differ; rerun with baseline dimensions/frames/warmup and the same style/path version".into());
    }
    for (scene, hash) in &new.meta.scene_hashes {
        if old.meta.scene_hashes.get(scene) != Some(hash) {
            return Err(format!("{scene}: scene hash mismatch or missing baseline scene; restore the scene or explicitly regenerate a reviewed baseline").into());
        }
        if old.meta.shadow_path.get(scene) != new.meta.shadow_path.get(scene) {
            return Err(format!(
                "{scene}: shadow_path changed; exact and fallback measurements cannot be compared"
            )
            .into());
        }
        for key in old
            .results
            .keys()
            .filter(|k| k.starts_with(&format!("{scene}/")))
        {
            if !new.results.contains_key(key) {
                return Err(format!("missing new phase {key}; rerun the complete scene").into());
            }
        }
    }
    for key in new.results.keys() {
        let (scene, _) = key
            .split_once('/')
            .ok_or_else(|| format!("invalid phase key {key}"))?;
        if !new.meta.scene_hashes.contains_key(scene) || !new.meta.shadow_path.contains_key(scene) {
            return Err(
                format!("{key}: missing scene hash or shadow_path; regenerate the report").into(),
            );
        }
    }
    if new.results.is_empty() {
        return Err("empty benchmark report".into());
    }
    Ok(())
}
pub fn compare(old: &BenchReport, new: &BenchReport, t: Tolerances) -> Result<(String, bool)> {
    compatible(old, new)?;
    let mut table = String::from(
        "| Scene/phase | Baseline p50 ms | New p50 ms | Wall Δ % | GPU Δ ms | Draw Δ | Triangle Δ | Upload Δ B | Vertex Δ B | Occupancy Δ B | Target Δ B | Device Δ B | RSS Δ MB | Result |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|\n",
    );
    let mut failed = false;
    for (key, phase) in &new.results {
        let base = old
            .results
            .get(key)
            .ok_or_else(|| format!("baseline lacks {key}; regenerate baseline for this phase"))?;
        let mut gates = vec![];
        for (metric, new_metric) in &phase.metrics {
            let new_value = new_metric.p50.median;
            if !new_value.is_finite() || new_value < 0.0 {
                return Err(format!("{key}: invalid {metric}").into());
            }
            let Some(old_value) = value(base, metric) else {
                continue;
            };
            if !old_value.is_finite() || old_value < 0.0 {
                return Err(format!("{key}: invalid baseline {metric}").into());
            }
            let gate = match metric.as_str() {
                "wall_ms" | "edit_to_pixels_ms" => {
                    regresses(old_value, new_value, t.wall_pct, t.wall_ms)
                }
                "gpu_total_ms" | "gpu_main_ms" | "gpu_resolve_ms" => {
                    regresses(old_value, new_value, t.gpu_pct, 0.0)
                }
                "draw_calls" | "triangles" | "upload_bytes" if key.ends_with("/fit") => {
                    new_value > old_value
                }
                "vertex_bytes"
                | "occupancy_bytes"
                | "target_bytes"
                | "device_allocated_bytes"
                | "rss_mb" => regresses(old_value, new_value, t.memory_pct, 0.0),
                _ => false,
            };
            if gate {
                gates.push(metric.clone());
            }
        }
        for metric in base.metrics.keys() {
            if !phase.metrics.contains_key(metric) {
                return Err(format!(
                    "{key}: metric {metric} disappeared; cannot validate regression gate"
                )
                .into());
            }
        }
        let before = value(base, "wall_ms").ok_or("baseline missing wall_ms")?;
        let after = value(phase, "wall_ms").ok_or("new report missing wall_ms")?;
        let delta = |metric: &str| match (value(base, metric), value(phase, metric)) {
            (Some(a), Some(b)) => format!("{:+.3}", b - a),
            _ => "n/a".into(),
        };
        let pct = if before > 0.0 {
            format!("{:+.2}", (after / before - 1.0) * 100.0)
        } else if after == 0.0 {
            "0.00".into()
        } else {
            "+∞".into()
        };
        let result = if gates.is_empty() {
            if after < before {
                "improved".into()
            } else {
                "pass".into()
            }
        } else {
            failed = true;
            format!("FAIL: {}", gates.join(", "))
        };
        writeln!(
            table,
            "| {key} | {before:.3} | {after:.3} | {pct} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {result} |",
            delta("gpu_total_ms"),
            delta("draw_calls"),
            delta("triangles"),
            delta("upload_bytes"),
            delta("vertex_bytes"),
            delta("occupancy_bytes"),
            delta("target_bytes"),
            delta("device_allocated_bytes"),
            delta("rss_mb")
        )?;
    }
    writeln!(
        table,
        "\nMachine: `{}`. Load (new): {:?} → {:?}. Per-run values/min/max are preserved in JSON.\n\nGates: wall/edit-to-pixels >{:.1}% AND >{:.3} ms; GPU >{:.1}%; fit draws/triangles/upload any increase; memory >{:.1}%.",
        new.meta.machine_key,
        new.meta.load_average.start,
        new.meta.load_average.end,
        t.wall_pct,
        t.wall_ms,
        t.gpu_pct,
        t.memory_pct
    )?;
    Ok((table, failed))
}
pub fn summary(report: &BenchReport) -> String {
    let mut out = String::from(
        "| Scene/phase | Wall p50 ms | p95 ms | FPS | p50 run min–max ms | GPU p50 ms |\n|---|---:|---:|---:|---:|---:|\n",
    );
    for (key, phase) in &report.results {
        if let Some(wall) = phase.metrics.get("wall_ms") {
            let gpu =
                value(phase, "gpu_total_ms").map_or_else(|| "n/a".into(), |v| format!("{v:.3}"));
            let _ = writeln!(
                out,
                "| {key} | {:.3} | {:.3} | {:.2} | {:.3}–{:.3} | {gpu} |",
                wall.p50.median, wall.p95.median, phase.fps_p50.median, wall.p50.min, wall.p50.max
            );
        }
    }
    let _ = writeln!(
        out,
        "\nMachine: `{}`. Load: {:?} → {:?}. Device initialization: {:.3} ms (excluded from cold).",
        report.meta.machine_key,
        report.meta.load_average.start,
        report.meta.load_average.end,
        report.meta.device_init_ms
    );
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        metrics::{Distribution, Metric},
        report::fixture,
    };
    #[test]
    fn wall_requires_both_relative_and_absolute_regression() {
        assert!(!regresses(1.0, 1.2, 10.0, 0.3));
        assert!(!regresses(1.0, 1.3, 10.0, 0.3));
        assert!(!regresses(10.0, 10.5, 10.0, 0.3));
        assert!(regresses(10.0, 11.1, 10.0, 0.3));
        assert!(!regresses(10.0, 8.0, 10.0, 0.3));
        assert!(regresses(0.0, 1.0, 10.0, 0.3));
    }
    #[test]
    fn compare_gates_and_compatibility() {
        let base = fixture();
        assert!(!compare(&base, &base, Tolerances::default()).unwrap().1);
        for metric in [
            "draw_calls",
            "triangles",
            "upload_bytes",
            "gpu_total_ms",
            "target_bytes",
            "wall_ms",
        ] {
            let mut new = base.clone();
            new.results.get_mut("autumn/fit").unwrap().metrics.insert(
                metric.into(),
                Metric::from_runs(&[Distribution::new(&[20.0]).unwrap()]).unwrap(),
            );
            assert!(
                compare(&base, &new, Tolerances::default()).unwrap().1,
                "{metric}"
            );
        }
        let mut new = base.clone();
        new.meta.machine_key = "different".into();
        assert!(compare(&base, &new, Tolerances::default()).is_err());
        let mut new = base.clone();
        new.meta
            .scene_hashes
            .insert("autumn".into(), "different".into());
        assert!(compare(&base, &new, Tolerances::default()).is_err());
        let mut new = base.clone();
        new.meta
            .shadow_path
            .insert("autumn".into(), "fallback".into());
        assert!(compare(&base, &new, Tolerances::default()).is_err());
    }
    #[test]
    fn comparison_refuses_lost_metrics_and_changed_sampling() {
        let base = fixture();
        let mut new = base.clone();
        new.results
            .get_mut("autumn/fit")
            .unwrap()
            .metrics
            .remove("gpu_total_ms");
        assert!(compare(&base, &new, Tolerances::default()).is_err());
        let mut new = base.clone();
        new.meta.frames += 1;
        assert!(compare(&base, &new, Tolerances::default()).is_err());
        let mut new = base.clone();
        new.meta.scene_hashes.clear();
        assert!(compare(&base, &new, Tolerances::default()).is_err());
    }
}
