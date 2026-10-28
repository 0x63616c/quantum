//! Screenshot comparison only, using the production resolve output before NV12.
use crate::{
    Result, block_on,
    cli::Options,
    paths,
    report::{self, Meta},
    scenes, style,
};
use serde::{Deserialize, Serialize};
use std::{
    io::BufReader,
    path::{Path, PathBuf},
    time::Instant,
};
use voxelite_core::Snapshot;
use voxelite_render::{
    Camera,
    gpu::{Headless, HeadlessOptions},
};

pub const DISPLAY: [u32; 2] = [640, 400];
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Diff {
    pub exact_match: bool,
    pub mean_abs_delta: f64,
    pub max_delta: u8,
    pub differing_pixels: usize,
    pub differing_pct: f64,
    /// None means infinity (exact equality), which JSON cannot represent numerically.
    pub psnr_db: Option<f64>,
}
impl Diff {
    pub fn fails(&self, max_pct: f64, max_delta: u8) -> bool {
        self.differing_pct > max_pct || self.max_delta > max_delta
    }
}
pub fn diff(golden: &[u8], actual: &[u8], delta: u8) -> Result<Diff> {
    if golden.is_empty() || golden.len() != actual.len() || !golden.len().is_multiple_of(4) {
        return Err("RGBA diff requires equal, nonempty RGBA8 images".into());
    }
    let mut total = 0u64;
    let mut squares = 0u64;
    let mut max_delta = 0;
    let mut differing_pixels = 0;
    for (a, b) in golden
        .as_chunks::<4>()
        .0
        .iter()
        .zip(actual.as_chunks::<4>().0)
    {
        let mut pixel = 0;
        for (a, b) in a.iter().zip(b) {
            let d = a.abs_diff(*b);
            pixel = pixel.max(d);
            max_delta = max_delta.max(d);
            total += u64::from(d);
            squares += u64::from(d).pow(2);
        }
        differing_pixels += usize::from(pixel > delta);
    }
    let mse = squares as f64 / golden.len() as f64;
    Ok(Diff {
        exact_match: total == 0,
        mean_abs_delta: total as f64 / golden.len() as f64,
        max_delta,
        differing_pixels,
        differing_pct: 100.0 * differing_pixels as f64 / (golden.len() / 4) as f64,
        psnr_db: if squares == 0 {
            None
        } else {
            Some(10.0 * ((255.0 * 255.0) / mse).log10())
        },
    })
}
pub fn heatmap(golden: &[u8], actual: &[u8]) -> Result<Vec<u8>> {
    diff(golden, actual, 0)?;
    Ok(golden
        .as_chunks::<4>()
        .0
        .iter()
        .zip(actual.as_chunks::<4>().0)
        .flat_map(|(g, a)| {
            let delta = g.iter().zip(a).map(|(g, a)| g.abs_diff(*a)).max().unwrap();
            if delta == 0 {
                *a
            } else {
                let gray = ((u32::from(a[0]) * 54 + u32::from(a[1]) * 183 + u32::from(a[2]) * 19)
                    / 1024) as u8;
                [gray.max(delta.saturating_mul(8)), gray, gray, 255]
            }
        })
        .collect())
}
pub fn side_by_side(size: [u32; 2], golden: &[u8], actual: &[u8], heat: &[u8]) -> Result<Vec<u8>> {
    let row = size[0] as usize * 4;
    let len = row.checked_mul(size[1] as usize).ok_or("image overflow")?;
    if row == 0
        || size[1] == 0
        || [golden.len(), actual.len(), heat.len()]
            .iter()
            .any(|&n| n != len)
    {
        return Err("side-by-side size mismatch".into());
    }
    let mut out = Vec::with_capacity(len * 3);
    for y in 0..size[1] as usize {
        for image in [golden, actual, heat] {
            out.extend_from_slice(&image[y * row..(y + 1) * row]);
        }
    }
    Ok(out)
}
pub fn write_png(path: &Path, size: [u32; 2], rgba: &[u8]) -> Result<()> {
    if u64::from(size[0]) * u64::from(size[1]) * 4 != rgba.len() as u64 {
        return Err("PNG size mismatch".into());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, size[0], size[1]);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(rgba)?;
    writer.finish()?;
    voxelite_format::storage::safe_write(path, &bytes)?;
    Ok(())
}
pub fn read_png(path: &Path) -> Result<([u32; 2], Vec<u8>)> {
    let decoder = png::Decoder::new(BufReader::new(std::fs::File::open(path)?));
    let mut reader = decoder.read_info()?;
    let len = reader
        .output_buffer_size()
        .filter(|&len| len <= 64 * 1024 * 1024)
        .ok_or("golden PNG exceeds 64 MiB decoded budget")?;
    let mut bytes = vec![0; len];
    let info = reader.next_frame(&mut bytes)?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err("golden must be RGBA8 PNG from this harness".into());
    }
    bytes.truncate(info.buffer_size());
    Ok(([info.width, info.height], bytes))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Golden {
    schema_version: u32,
    machine_key: String,
    git_commit: String,
    date: String,
    scene_hash: String,
    style: String,
    msaa: u32,
    supersample: u32,
    size: [u32; 2],
    shadow_path: String,
}
impl Golden {
    fn compatible(&self, other: &Self) -> Result<()> {
        if self.schema_version != 1 {
            return Err("unsupported golden metadata schema; explicitly rebaseline".into());
        }
        if self.machine_key != other.machine_key {
            return Err("golden metadata machine_key differs from selected golden set".into());
        }
        if self.scene_hash != other.scene_hash {
            return Err("scene hash changed; restore the original scene or explicitly rebaseline and review the side-by-side".into());
        }
        if self.style != other.style
            || self.msaa != other.msaa
            || self.supersample != other.supersample
            || self.size != other.size
        {
            return Err("golden render settings changed; comparison is invalid".into());
        }
        if self.shadow_path != other.shadow_path {
            return Err(
                "shadow_path changed; exact and fallback images cannot share a golden".into(),
            );
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub scene: String,
    pub still: String,
    pub status: String,
    pub error: Option<String>,
    pub diff: Option<Diff>,
    pub deterministic: Option<bool>,
    pub actual: Option<String>,
    pub heatmap: Option<String>,
    pub side_by_side: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VisualReport {
    pub schema_version: u32,
    pub meta: Meta,
    pub golden_set: String,
    pub delta: u8,
    pub max_pct: f64,
    pub max_delta: u8,
    pub rebaseline: bool,
    pub passed: bool,
    pub changed_files: Vec<String>,
    pub results: Vec<Entry>,
}
fn artifacts(
    out: &Path,
    entry: &mut Entry,
    size: [u32; 2],
    actual: &[u8],
    golden: Option<&[u8]>,
) -> Result<()> {
    let name = format!("{}_{}", entry.scene, entry.still);
    let actual_name = format!("{name}_actual.png");
    write_png(&out.join(&actual_name), size, actual)?;
    entry.actual = Some(actual_name);
    if let Some(golden) = golden {
        let heat = heatmap(golden, actual)?;
        let heat_name = format!("{name}_diff.png");
        write_png(&out.join(&heat_name), size, &heat)?;
        entry.heatmap = Some(heat_name);
        let side = side_by_side(size, golden, actual, &heat)?;
        let side_name = format!("{name}_side-by-side.png");
        write_png(&out.join(&side_name), [size[0] * 3, size[1]], &side)?;
        entry.side_by_side = Some(side_name);
    }
    Ok(())
}
fn fresh(snapshot: &Snapshot, camera: Camera) -> Result<voxelite_render::gpu::HeadlessFrame> {
    let mut gpu = block_on(Headless::new(HeadlessOptions::default()))?;
    Ok(block_on(gpu.render(
        Some(snapshot.clone()),
        snapshot.bounds,
        camera,
        DISPLAY,
        style(snapshot.bounds),
        true,
    ))?)
}
pub fn run(options: &Options) -> Result<bool> {
    let start = Instant::now();
    let mut gpu = block_on(Headless::new(HeadlessOptions::default()))?;
    let mut meta = Meta::new(
        gpu.adapter_info(),
        DISPLAY,
        1,
        0,
        1,
        start.elapsed().as_secs_f64() * 1000.0,
    )?;
    let pinned = report::pinned_machine()?;
    let key = options
        .golden_set
        .clone()
        .unwrap_or_else(|| meta.machine_key.clone());
    if options.rebaseline && !options.local_baseline && meta.machine_key != pinned && !options.force
    {
        return Err(format!("rebaseline refused: machine {} is not pinned {}; use --local-baseline for this machine, or explicitly --force",meta.machine_key,pinned).into());
    }
    if !options.rebaseline
        && !options.local_baseline
        && options.golden_set.is_none()
        && meta.machine_key != pinned
    {
        return Err(format!("unpinned machine {}; pinned set is {pinned}. Use --local-baseline for same-machine before/after checks, or --golden-set {pinned} explicitly for cross-machine diagnosis",meta.machine_key).into());
    }
    let dir = PathBuf::from(if options.local_baseline {
        "target/golden"
    } else {
        "bench/golden"
    })
    .join(&key);
    // Only an entirely absent local set is initialized implicitly. Missing files in
    // an existing set fail, catching accidental deletions and incomplete baselines.
    let initialize = options.local_baseline && !dir.exists();
    let update = options.rebaseline || initialize;
    if initialize {
        eprintln!(
            "Creating explicit local baseline at {}; subsequent runs compare it",
            dir.display()
        );
    }
    std::fs::create_dir_all(&options.out)?;
    let mut entries = vec![];
    let mut changed = vec![];
    let mut passed = true;
    for scene in &options.scenes {
        let snapshot = scenes::load(scene)?;
        let hash = scenes::content_hash(&snapshot)?;
        meta.scene_hashes.insert(scene.clone(), hash.clone());
        meta.scene_voxels.insert(
            scene.clone(),
            snapshot.layers.iter().map(|l| l.voxels.len()).sum(),
        );
        let fit = paths::fit(&snapshot, DISPLAY)?;
        gpu.reset_scene()?;
        for (i, (still, camera)) in paths::stills(scene, fit).into_iter().enumerate() {
            let mut entry = Entry {
                scene: scene.clone(),
                still: still.into(),
                status: "pass".into(),
                error: None,
                diff: None,
                deterministic: None,
                actual: None,
                heatmap: None,
                side_by_side: None,
            };
            let rendered = if options.determinism {
                fresh(&snapshot, camera)
            } else {
                block_on(gpu.render(
                    (i == 0).then(|| snapshot.clone()),
                    snapshot.bounds,
                    camera,
                    DISPLAY,
                    style(snapshot.bounds),
                    true,
                ))
                .map_err(Into::into)
            };
            let frame = match rendered {
                Ok(frame) => frame,
                Err(error) => {
                    entry.status = "error".into();
                    entry.error = Some(error.to_string());
                    passed = false;
                    entries.push(entry);
                    continue;
                }
            };
            let actual = frame.rgba.ok_or("capture returned no pixels")?;
            let shadow = frame.stats.shadow_path.as_str().to_owned();
            meta.shadow_path.insert(scene.clone(), shadow.clone());
            if options.determinism {
                match fresh(&snapshot, camera) {
                    Ok(other)
                        if other.size == frame.size && other.rgba.as_deref() == Some(&actual) =>
                    {
                        entry.deterministic = Some(true)
                    }
                    result => {
                        entry.deterministic = Some(false);
                        entry.status = "NONDETERMINISTIC".into();
                        passed = false;
                        entry.error=Some(match &result {Err(e)=>format!("determinism render failed: {e}"),_=>"GPU NONDETERMINISM: fresh renderers produced different bytes; goldens cannot be trusted".into()});
                        let other = result
                            .as_ref()
                            .ok()
                            .filter(|f| f.size == frame.size)
                            .and_then(|f| f.rgba.as_deref());
                        artifacts(&options.out, &mut entry, frame.size, &actual, other)?;
                        eprintln!("{scene}/{still}: {}", entry.error.as_deref().unwrap());
                        entries.push(entry);
                        continue;
                    }
                }
            }
            let golden_path = dir.join(format!("{scene}_{still}.png"));
            let meta_path = golden_path.with_extension("json");
            let current = Golden {
                schema_version: 1,
                machine_key: key.clone(),
                git_commit: meta.git_commit.clone(),
                date: meta.date.clone(),
                scene_hash: hash.clone(),
                style: meta.style.clone(),
                msaa: 4,
                supersample: 2,
                size: frame.size,
                shadow_path: shadow,
            };
            let old_meta = report::read_json::<Golden>(&meta_path);
            let old = if golden_path.exists() {
                Some(read_png(&golden_path))
            } else {
                None
            };
            let valid_pixels = old
                .as_ref()
                .and_then(|r| r.as_ref().ok())
                .filter(|(size, _)| *size == frame.size)
                .map(|(_, p)| p.as_slice());
            if let Some(pixels) = valid_pixels {
                entry.diff = Some(diff(pixels, &actual, options.delta)?);
            }
            if update {
                let pixels_changed = entry.diff.as_ref().is_none_or(|d| !d.exact_match);
                let metadata_changed = !old_meta
                    .as_ref()
                    .is_ok_and(|g| g.compatible(&current).is_ok());
                if pixels_changed || metadata_changed {
                    artifacts(&options.out, &mut entry, frame.size, &actual, valid_pixels)?;
                    write_png(&golden_path, frame.size, &actual)?;
                    report::write_json(&meta_path, &current)?;
                    changed.push(golden_path.display().to_string());
                    changed.push(meta_path.display().to_string());
                    entry.status = "rebaselined".into();
                } else {
                    entry.status = "unchanged".into();
                }
            } else {
                let issue = if old.is_none() {
                    Some(format!(
                        "missing golden {}; create it on {pinned} with cargo xtask visual --rebaseline --scene {scene} (local sets: add --local-baseline)",
                        golden_path.display()
                    ))
                } else if let Some(Err(e)) = &old {
                    Some(format!("cannot read golden: {e}"))
                } else if valid_pixels.is_none() {
                    Some("golden image dimensions differ".into())
                } else {
                    match old_meta {
                        Ok(old) => old.compatible(&current).err().map(|e| e.to_string()),
                        Err(e) => Some(format!(
                            "missing/invalid golden metadata: {e}; explicitly rebaseline"
                        )),
                    }
                };
                if let Some(issue) = issue {
                    entry.status = "invalid".into();
                    entry.error = Some(issue);
                } else if entry
                    .diff
                    .as_ref()
                    .is_some_and(|d| d.fails(options.max_pct, options.max_delta))
                {
                    entry.status = "FAIL".into();
                    entry.error = Some("pixel difference exceeds tolerance".into());
                }
                if entry.error.is_some() {
                    passed = false;
                    artifacts(&options.out, &mut entry, frame.size, &actual, valid_pixels)?;
                }
            }
            eprintln!(
                "{scene}/{still}: {}{}{}",
                entry.status,
                entry.diff.as_ref().map_or_else(String::new, |d| format!(
                    "; exact={}, max Δ {}, differing {:.6}%",
                    d.exact_match, d.max_delta, d.differing_pct
                )),
                entry
                    .error
                    .as_ref()
                    .map_or_else(String::new, |e| format!("; {e}"))
            );
            entries.push(entry);
        }
    }
    meta.load_average.end = report::load_average();
    let report = VisualReport {
        schema_version: 1,
        meta,
        golden_set: key,
        delta: options.delta,
        max_pct: options.max_pct,
        max_delta: options.max_delta,
        rebaseline: update,
        passed,
        changed_files: changed,
        results: entries,
    };
    report::write_json(&options.out.join("report.json"), &report)?;
    voxelite_format::storage::safe_write(options.out.join("report.md"), markdown(&report))?;
    for path in &report.changed_files {
        println!("changed: {path}");
    }
    println!(
        "Visual {}. Report: {}",
        if passed { "passed" } else { "FAILED" },
        options.out.join("report.md").display()
    );
    Ok(passed)
}
fn markdown(report: &VisualReport) -> String {
    use std::fmt::Write;
    let mut out = format!(
        "Visual {} — machine `{}`, golden set `{}`. Delta >{}, max differing {}%, max delta {}.\n\n| Scene/still | Status | Exact | Mean abs Δ | Max Δ | Differing % | PSNR dB | Artifacts |\n|---|---|---|---:|---:|---:|---:|---|\n",
        if report.passed { "passed" } else { "FAILED" },
        report.meta.machine_key,
        report.golden_set,
        report.delta,
        report.max_pct,
        report.max_delta
    );
    for entry in &report.results {
        let metrics = entry.diff.as_ref().map_or_else(
            || "— | — | — | — | —".into(),
            |d| {
                format!(
                    "{} | {:.6} | {} | {:.6} | {}",
                    d.exact_match,
                    d.mean_abs_delta,
                    d.max_delta,
                    d.differing_pct,
                    d.psnr_db.map_or_else(|| "∞".into(), |v| format!("{v:.3}"))
                )
            },
        );
        let links = [
            ("actual", &entry.actual),
            ("heatmap", &entry.heatmap),
            ("side-by-side", &entry.side_by_side),
        ]
        .iter()
        .filter_map(|(label, p)| p.as_ref().map(|p| format!("[{label}]({p})")))
        .collect::<Vec<_>>()
        .join(" · ");
        let _ = writeln!(
            out,
            "| {}/{} | {} | {metrics} | {links} |",
            entry.scene, entry.still, entry.status
        );
    }
    for entry in &report.results {
        if let Some(error) = &entry.error {
            let _ = writeln!(out, "\n{}/{}: {error}", entry.scene, entry.still);
        }
    }
    if report.rebaseline {
        out.push_str("\nReview every changed side-by-side before accepting these goldens.\n");
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diff_math_thresholds_and_single_pixel_peak() {
        let base = [0, 0, 0, 255, 100, 100, 100, 255];
        let exact = diff(&base, &base, 2).unwrap();
        assert!(exact.exact_match);
        assert_eq!(exact.psnr_db, None);
        let actual = [2, 0, 0, 255, 100, 104, 100, 255];
        let d = diff(&base, &actual, 2).unwrap();
        assert_eq!(d.max_delta, 4);
        assert_eq!(d.differing_pixels, 1);
        assert_eq!(d.differing_pct, 50.0);
        assert_eq!(d.mean_abs_delta, 0.75);
        assert!((d.psnr_db.unwrap() - 10.0 * ((255.0f64 * 255.0) / 2.5).log10()).abs() < 1e-10);
        assert!(!d.fails(50.0, 4));
        assert!(d.fails(49.0, 4));
        assert!(d.fails(100.0, 3));
        let base = vec![0; 4 * 20_000];
        let mut actual = base.clone();
        actual[0] = 25;
        assert!(diff(&base, &actual, 2).unwrap().fails(0.01, 24));
        assert!(diff(&[], &[], 2).is_err());
        assert!(diff(&[0], &[0], 2).is_err());
    }
    #[test]
    fn heatmap_preserves_equal_pixels_and_amplifies_changed_pixels() {
        let golden = [40, 80, 120, 255, 100, 100, 100, 255];
        let actual = [40, 80, 120, 255, 110, 100, 100, 255];
        let heat = heatmap(&golden, &actual).unwrap();
        assert_eq!(&heat[..4], &actual[..4]);
        assert_eq!(heat[4], 80);
        assert_eq!(heat[5], heat[6]);
        assert_eq!(heat[7], 255);
        assert_eq!(
            side_by_side([1, 2], &golden, &actual, &heat).unwrap()[..12],
            [40, 80, 120, 255, 40, 80, 120, 255, 40, 80, 120, 255]
        );
    }
    #[test]
    fn golden_metadata_rejects_scene_or_quality_drift() {
        let reference = Golden {
            schema_version: 1,
            machine_key: "test".into(),
            git_commit: "test".into(),
            date: "test".into(),
            scene_hash: "original".into(),
            style: crate::STYLE_ID.into(),
            msaa: 4,
            supersample: 2,
            size: [1280, 800],
            shadow_path: "exact".into(),
        };
        let mut current = reference.clone();
        current.git_commit = "later".into();
        assert!(reference.compatible(&current).is_ok());
        current.scene_hash = "different".into();
        assert!(
            reference
                .compatible(&current)
                .unwrap_err()
                .to_string()
                .contains("scene hash")
        );
        let mut current = reference.clone();
        current.msaa = 1;
        assert!(reference.compatible(&current).is_err());
        let mut current = reference.clone();
        current.shadow_path = "fallback".into();
        assert!(reference.compatible(&current).is_err());
    }
}
