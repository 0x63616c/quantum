//! Stable schema (struct field order + BTreeMaps), machine and process metadata.
use crate::{
    Result, STYLE_ID,
    metrics::{Metric, Spread},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, process::Command};
use voxelite_render::gpu::AdapterInfo;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Adapter {
    pub name: String,
    pub backend: String,
    pub driver: String,
    pub driver_info: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Load {
    pub start: Option<[f64; 3]>,
    pub end: Option<[f64; 3]>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub git_commit: String,
    pub dirty: bool,
    pub date: String,
    pub os: String,
    pub hw_model: String,
    pub chip: String,
    pub adapter: Adapter,
    pub machine_key: String,
    pub display_size: [u32; 2],
    pub surface_size: [u32; 2],
    pub msaa: u32,
    pub supersample: u32,
    pub style: String,
    pub paths_version: u32,
    pub frames: usize,
    pub warmup: usize,
    pub runs: usize,
    pub device_init_ms: f64,
    pub load_average: Load,
    pub run_load: Vec<Load>,
    pub scene_hashes: BTreeMap<String, String>,
    pub scene_voxels: BTreeMap<String, usize>,
    pub shadow_path: BTreeMap<String, String>,
}
impl Meta {
    pub fn new(
        info: &AdapterInfo,
        display: [u32; 2],
        frames: usize,
        warmup: usize,
        runs: usize,
        init: f64,
    ) -> Result<Self> {
        let git_commit = std::env::var("GITHUB_SHA")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| command("git", &["rev-parse", "HEAD"]))
            .ok_or("cannot determine git commit; run in the repository or set GITHUB_SHA")?;
        let status =
            command("git", &["status", "--porcelain"]).ok_or("cannot determine git dirty flag")?;
        let (hw_model, chip) = hardware();
        let machine_key = machine_key(&chip, info);
        Ok(Self {
            git_commit,
            dirty: !status.is_empty(),
            date: command("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]).ok_or("cannot read UTC date")?,
            os: std::env::consts::OS.into(),
            hw_model,
            chip,
            adapter: Adapter {
                name: info.name.clone(),
                backend: info.backend.clone(),
                driver: info.driver.clone(),
                driver_info: info.driver_info.clone(),
            },
            machine_key,
            display_size: display,
            surface_size: [
                display[0].checked_mul(2).ok_or("width overflow")?,
                display[1].checked_mul(2).ok_or("height overflow")?,
            ],
            msaa: 4,
            supersample: 2,
            style: STYLE_ID.into(),
            paths_version: 1,
            frames,
            warmup,
            runs,
            device_init_ms: init,
            load_average: Load {
                start: load_average(),
                end: None,
            },
            run_load: vec![],
            scene_hashes: BTreeMap::new(),
            scene_voxels: BTreeMap::new(),
            shadow_path: BTreeMap::new(),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Phase {
    pub samples_per_run: Vec<usize>,
    pub fps_p50: Spread,
    pub metrics: BTreeMap<String, Metric>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BenchReport {
    pub schema_version: u32,
    pub meta: Meta,
    pub results: BTreeMap<String, Phase>,
}

pub fn command(program: &str, args: &[&str]) -> Option<String> {
    let result = Command::new(program).args(args).output().ok()?;
    result
        .status
        .success()
        .then(|| String::from_utf8_lossy(&result.stdout).trim().to_owned())
}
fn hardware() -> (String, String) {
    if cfg!(target_os = "macos") {
        (
            command("sysctl", &["-n", "hw.model"]).unwrap_or_else(|| "unknown".into()),
            command("sysctl", &["-n", "machdep.cpu.brand_string"])
                .unwrap_or_else(|| "unknown".into()),
        )
    } else {
        let cpu = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let chip = cpu
            .lines()
            .find_map(|l| {
                l.strip_prefix("model name")
                    .and_then(|s| s.split_once(':').map(|(_, v)| v.trim().to_owned()))
            })
            .unwrap_or_else(|| std::env::consts::ARCH.into());
        (std::env::consts::ARCH.into(), chip)
    }
}
pub fn slug(text: &str) -> String {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
pub fn machine_key(chip: &str, info: &AdapterInfo) -> String {
    let name = info.name.to_lowercase();
    let device = if name.contains("llvmpipe") {
        "llvmpipe".into()
    } else if name.contains("lavapipe") {
        "lavapipe".into()
    } else if info.backend == "metal" {
        slug(chip)
    } else {
        slug(&info.name)
    };
    format!("{device}-{}", info.backend)
}
pub fn load_average() -> Option<[f64; 3]> {
    let text = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .or_else(|| command("sysctl", &["-n", "vm.loadavg"]))?;
    let nums: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .take(3)
        .collect();
    nums.try_into().ok()
}
pub fn rss_mb() -> Option<f64> {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        return status.lines().find_map(|l| {
            l.strip_prefix("VmRSS:")
                .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
                .map(|kb| kb / 1024.0)
        });
    }
    command("ps", &["-o", "rss=", "-p", &std::process::id().to_string()])?
        .parse::<f64>()
        .ok()
        .map(|kb| kb / 1024.0)
}
pub fn write_json(path: &Path, report: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut data = serde_json::to_vec_pretty(report)?;
    data.push(b'\n');
    // Complete serialization before touching the old file.
    voxelite_format::storage::safe_write(path, data)?;
    Ok(())
}
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(
        &std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?,
    )?)
}
pub fn pinned_machine() -> Result<String> {
    let text = std::fs::read_to_string("bench/pinned-machine.toml")?;
    let value = text
        .lines()
        .find_map(|l| {
            l.split_once('=')
                .filter(|(k, _)| k.trim() == "machine_key")
                .map(|(_, v)| v.trim().trim_matches('"').to_owned())
        })
        .ok_or("bench/pinned-machine.toml needs machine_key = \"...\"")?;
    if slug(&value) != value || value.is_empty() {
        return Err("invalid pinned machine key".into());
    }
    Ok(value)
}

#[cfg(test)]
pub(crate) fn fixture() -> BenchReport {
    use crate::metrics::Distribution;
    BenchReport {
        schema_version: 1,
        meta: Meta {
            git_commit: "test".into(),
            dirty: false,
            date: "2026-10-26T00:00:00Z".into(),
            os: "test".into(),
            hw_model: "test".into(),
            chip: "test".into(),
            adapter: Adapter {
                name: "test".into(),
                backend: "test".into(),
                driver: "test".into(),
                driver_info: "test".into(),
            },
            machine_key: "test".into(),
            display_size: [640, 400],
            surface_size: [1280, 800],
            msaa: 4,
            supersample: 2,
            style: STYLE_ID.into(),
            paths_version: 1,
            frames: 90,
            warmup: 8,
            runs: 1,
            device_init_ms: 1.0,
            load_average: Load {
                start: None,
                end: None,
            },
            run_load: vec![],
            scene_hashes: BTreeMap::from([("autumn".into(), "hash".into())]),
            scene_voxels: BTreeMap::from([("autumn".into(), 1)]),
            shadow_path: BTreeMap::from([("autumn".into(), "exact".into())]),
        },
        results: BTreeMap::from([(
            "autumn/fit".into(),
            Phase {
                samples_per_run: vec![90],
                fps_p50: Spread::new(vec![100.0]).unwrap(),
                metrics: [
                    "wall_ms",
                    "gpu_total_ms",
                    "draw_calls",
                    "triangles",
                    "upload_bytes",
                    "target_bytes",
                ]
                .iter()
                .map(|k| {
                    (
                        k.to_string(),
                        Metric::from_runs(&[Distribution::new(&[10.0]).unwrap()]).unwrap(),
                    )
                })
                .collect(),
            },
        )]),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_roundtrip_and_key_order() {
        let report = fixture();
        let bytes = serde_json::to_vec_pretty(&report).unwrap();
        assert_eq!(
            report,
            serde_json::from_slice::<BenchReport>(&bytes).unwrap()
        );
        assert_eq!(
            bytes,
            serde_json::to_vec_pretty(&serde_json::from_slice::<BenchReport>(&bytes).unwrap())
                .unwrap()
        );
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .starts_with("{\n  \"schema_version\": 1,")
        );
    }
    #[test]
    fn stable_machine_keys() {
        let mut info = AdapterInfo {
            name: "Apple M2".into(),
            backend: "metal".into(),
            driver: String::new(),
            driver_info: String::new(),
        };
        assert_eq!(machine_key("Apple M2", &info), "apple-m2-metal");
        assert_eq!(machine_key("Apple M5 Pro", &info), "apple-m5-pro-metal");
        info.name = "llvmpipe (LLVM 19)".into();
        info.backend = "vulkan".into();
        assert_eq!(machine_key("cpu", &info), "llvmpipe-vulkan");
    }
}
