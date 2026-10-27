//! Strict, dependency-free CLI parsing. Bench and visual flag sets are independent.
use crate::{Result, scenes::DEFAULT_SCENES};
use std::path::PathBuf;
#[derive(Clone, Debug)]
pub struct Options {
    pub command: String,
    pub scenes: Vec<String>,
    pub frames: usize,
    pub warmup: usize,
    pub runs: usize,
    pub display: [u32; 2],
    pub out: PathBuf,
    pub compare: Option<Option<PathBuf>>,
    pub input: Option<PathBuf>,
    pub update_baseline: bool,
    pub markdown: Option<PathBuf>,
    pub wall_pct: f64,
    pub wall_ms: f64,
    pub gpu_pct: f64,
    pub memory_pct: f64,
    pub delta: u8,
    pub max_pct: f64,
    pub max_delta: u8,
    pub rebaseline: bool,
    pub force: bool,
    pub determinism: bool,
    pub golden_set: Option<String>,
    pub local_baseline: bool,
}
impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter().peekable();
        let command = args.next().unwrap_or_else(|| "help".into());
        if !["bench", "visual", "help", "--help", "-h"].contains(&command.as_str()) {
            return Err(format!("unknown command {command}; use help").into());
        }
        let mut o = Self {
            out: if command == "visual" {
                "target/visual".into()
            } else {
                "target/bench/latest.json".into()
            },
            command,
            scenes: vec![],
            frames: 90,
            warmup: 8,
            runs: 1,
            display: [2064, 1616],
            compare: None,
            input: None,
            update_baseline: false,
            markdown: None,
            wall_pct: 10.0,
            wall_ms: 0.3,
            gpu_pct: 10.0,
            memory_pct: 10.0,
            delta: 2,
            max_pct: 0.01,
            max_delta: 24,
            rebaseline: false,
            force: false,
            determinism: false,
            golden_set: None,
            local_baseline: false,
        };
        while let Some(arg) = args.next() {
            let bench = o.command == "bench";
            let visual = o.command == "visual";
            let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
            match arg.as_str() {
                "--help" | "-h" => {
                    o.command = "help".into();
                    return Ok(o);
                }
                "--scene" => o.scenes.push(value()?),
                "--out" => o.out = value()?.into(),
                "--frames" if bench => o.frames = value()?.parse()?,
                "--warmup" if bench => o.warmup = value()?.parse()?,
                "--runs" if bench => o.runs = value()?.parse()?,
                "--width" if bench => o.display[0] = value()?.parse()?,
                "--height" if bench => o.display[1] = value()?.parse()?,
                "--input" if bench => o.input = Some(value()?.into()),
                "--markdown" if bench => o.markdown = Some(value()?.into()),
                "--update-baseline" if bench => o.update_baseline = true,
                "--wall-pct" if bench => o.wall_pct = value()?.parse()?,
                "--wall-ms" if bench => o.wall_ms = value()?.parse()?,
                "--gpu-pct" if bench => o.gpu_pct = value()?.parse()?,
                "--memory-pct" if bench => o.memory_pct = value()?.parse()?,
                "--compare" if bench => {
                    o.compare = Some(if args.peek().is_some_and(|s| !s.starts_with('-')) {
                        args.next().map(PathBuf::from)
                    } else {
                        None
                    })
                }
                "--delta" if visual => o.delta = value()?.parse()?,
                "--max-pct" if visual => o.max_pct = value()?.parse()?,
                "--max-delta" if visual => o.max_delta = value()?.parse()?,
                "--rebaseline" if visual => o.rebaseline = true,
                "--force" if visual => o.force = true,
                "--determinism" if visual => o.determinism = true,
                "--golden-set" if visual => o.golden_set = Some(value()?),
                "--local-baseline" if visual => o.local_baseline = true,
                _ => return Err(format!("unknown {} option {arg:?}; use help", o.command).into()),
            }
        }
        if o.scenes.is_empty() {
            o.scenes = DEFAULT_SCENES.iter().map(|s| s.to_string()).collect();
        }
        o.scenes.sort();
        o.scenes.dedup();
        for s in &o.scenes {
            if !["autumn", "terrain1m", "sponge1m", "terrain10m"].contains(&s.as_str()) {
                return Err(format!("unknown scene {s}").into());
            }
        }
        if o.frames == 0
            || o.runs == 0
            || o.display.contains(&0)
            || o.frames.checked_add(o.warmup).is_none()
        {
            return Err(
                "frames, runs and dimensions must be positive; frames + warmup must fit usize"
                    .into(),
            );
        }
        for v in [o.wall_pct, o.wall_ms, o.gpu_pct, o.memory_pct, o.max_pct] {
            if !v.is_finite() || v < 0.0 {
                return Err("tolerances must be finite and nonnegative".into());
            }
        }
        if o.max_pct > 100.0 {
            return Err("max-pct must be at most 100".into());
        }
        if o.local_baseline && o.golden_set.is_some() {
            return Err("--local-baseline and --golden-set are mutually exclusive".into());
        }
        if let Some(key) = &o.golden_set
            && (key.is_empty() || crate::report::slug(key) != *key)
        {
            return Err("golden-set must be a lowercase machine slug".into());
        }
        if o.force && !o.rebaseline {
            return Err("--force requires --rebaseline".into());
        }
        if o.rebaseline && o.golden_set.is_some() {
            return Err(
                "rebaseline writes only the current machine's set; omit --golden-set".into(),
            );
        }
        if o.update_baseline && (o.compare.is_some() || o.input.is_some()) {
            return Err("--update-baseline requires a fresh run without --compare/--input".into());
        }
        Ok(o)
    }
}
pub const HELP: &str = "Rendering harness (production 4x MSAA, 2x supersampling, AO and shadow path)\n\
  cargo xtask bench [--scene S ...] [--frames 90] [--warmup 8] [--runs N]\n\
    [--out FILE] [--compare [FILE]] [--update-baseline] [--markdown FILE]\n\
    [--width 2064 --height 1616] [--wall-pct 10 --wall-ms 0.3 --gpu-pct 10 --memory-pct 10]\n\
    [--input FILE] (compare/report existing results without rerunning)\n\
  cargo xtask visual [--scene S ...] [--out DIR] [--determinism]\n\
    [--rebaseline [--force]] [--local-baseline | --golden-set KEY]\n\
    [--delta 2 --max-pct 0.01 --max-delta 24]\n\
Scenes: autumn, terrain1m, sponge1m; terrain10m opts into bench-only large-scenes.\n\
Default bench baseline: bench/baseline/<machine_key>.json.\n\
Visual display is fixed at 640x400; capture is 1280x800 RGBA8 sRGB.\n\
Local visual baseline: first invocation creates target/golden/<machine_key>; later invocations compare.\n\
See docs/perf/rendering-harness.md.";

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<Options> {
        Options::parse(args.iter().map(|s| s.to_string()))
    }
    #[test]
    fn commands_have_independent_flags_and_reject_invalid_limits() {
        for args in [
            vec!["bench", "--determinism"],
            vec!["visual", "--runs", "2"],
            vec!["bench", "--frames", "0"],
            vec!["bench", "--runs", "0"],
            vec!["visual", "--max-pct", "NaN"],
            vec!["visual", "--golden-set", "../outside"],
            vec!["bench", "--compare", "--update-baseline"],
        ] {
            assert!(parse(&args).is_err(), "{args:?}");
        }
        let options = parse(&["bench", "--compare", "--scene", "autumn"]).unwrap();
        assert_eq!(options.compare, Some(None));
        assert_eq!(options.scenes, vec!["autumn"]);
        let options = parse(&["bench", "--compare", "baseline.json"]).unwrap();
        assert_eq!(options.compare, Some(Some("baseline.json".into())));
    }
}
