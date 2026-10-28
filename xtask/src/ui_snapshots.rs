//! Strict native UI golden gate, sharing launch states with PR screenshots.
use std::{fs, path::Path};

use xshell::{Shell, cmd};
use xtask::cmd::{Command, Runner, SystemRunner};

const USAGE: &str = "cargo xtask ui-snapshots [--rebaseline]\n\
Capture key screens at native 1x and 2x on the golden Mac. Missing captures or\n\
goldens fail. Only --rebaseline writes bench/ui-golden/macos/. See docs/ui-snapshots.md.";
const MAX_CHANGED_FRACTION: f64 = 0.0005;

pub fn run(args: &[String]) -> Result<(), String> {
    if args == ["--help"] || args == ["-h"] {
        println!("{USAGE}");
        return Ok(());
    }
    let rebaseline = match args {
        [] => false,
        [arg] if arg == "--rebaseline" => true,
        _ => return Err(USAGE.into()),
    };
    if !cfg!(target_os = "macos") {
        return Err("native UI goldens require the golden Mac's logged-in GUI session; Linux PR screenshots are not native goldens".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("workspace root")?;
    let out = root.join("target/ui-snapshots");
    // Do not remove previous output: each run owns its temporary capture directory.
    fs::create_dir_all(out.join("bin")).map_err(|e| e.to_string())?;
    let capture = tempfile::Builder::new()
        .prefix("capture-")
        .tempdir_in(&out)
        .map_err(|e| e.to_string())?;
    let actual = capture.path().join("actual");
    fs::create_dir(&actual).map_err(|e| e.to_string())?;
    let spec = crate::screenshots::golden_spec();
    let spec_path = capture.path().join("screens.tsv");
    fs::write(&spec_path, &spec).map_err(|e| e.to_string())?;
    let binary = crate::screenshots::build(root, &out, "ui-snapshots")?;
    let script = root.join("scripts/capture-ui-snapshots.sh");
    let sh = Shell::new().map_err(|e| e.to_string())?;
    let runner = SystemRunner::default();
    let command: Command = cmd!(sh, "bash {script} {binary} {actual} {spec_path}").into();
    let captured = command.run(&runner);
    // Keep captures and diffs even on failure for CI artifacts and local inspection.
    let retained = capture.keep();
    captured.map_err(|e| format!("{e}; captures: {}", retained.display()))?;
    let golden = root.join("bench/ui-golden/macos");
    let mut failures = Vec::new();
    let mut files = Vec::new();
    for line in spec.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        let width: u32 = columns[2].parse().map_err(|_| "invalid registry width")?;
        let height: u32 = columns[3].parse().map_err(|_| "invalid registry height")?;
        for scale in [1, 2] {
            let name = format!("{}-{scale}x.png", columns[0]);
            let image = actual.join(&name);
            let expected = golden.join(&name);
            let command: Command = cmd!(sh, "identify -format '%w %h' {image}").into();
            let dimensions = command.read(&runner).map_err(|e| e.to_string())?;
            if dimensions.trim() != format!("{} {}", width * scale, height * scale) {
                failures.push(format!("{name}: wrong native pixel size: {}", dimensions));
                continue;
            }
            files.push((image.clone(), expected.clone()));
            if rebaseline {
                continue;
            }
            if !expected.is_file() {
                failures.push(format!(
                    "{name}: missing golden (explicit rebaseline required)"
                ));
                continue;
            }
            let command: Command = cmd!(sh, "identify -format '%w %h' {expected}").into();
            let dimensions = command.read(&runner).map_err(|e| e.to_string())?;
            if dimensions.trim() != format!("{} {}", width * scale, height * scale) {
                failures.push(format!("{name}: golden dimensions differ"));
                continue;
            }
            let diff = retained.join(format!("diff-{name}"));
            let command: Command =
                cmd!(sh, "compare -metric AE -fuzz 2% {expected} {image} {diff}").into();
            let result = runner.execute(&command, true).map_err(|e| e.to_string())?;
            match accepted(
                result.status.code(),
                &String::from_utf8_lossy(&result.stderr),
                width * height * scale * scale,
            ) {
                Ok(true) => {}
                Ok(false) => failures.push(format!(
                    "{name}: unexpected visual change ({})",
                    String::from_utf8_lossy(&result.stderr).trim()
                )),
                Err(error) => failures.push(format!("{name}: {error}")),
            }
        }
    }
    let report = if failures.is_empty() {
        "All key screens passed.\n".into()
    } else {
        failures.join("\n")
    };
    fs::write(retained.join("report.txt"), &report).map_err(|e| e.to_string())?;
    if !failures.is_empty() {
        return Err(format!("{report}\nArtifacts: {}", retained.display()));
    }
    if rebaseline {
        // Validate the complete capture set before touching a single reference.
        fs::create_dir_all(&golden).map_err(|e| e.to_string())?;
        for (image, expected) in files {
            fs::copy(image, expected).map_err(|e| e.to_string())?;
        }
    }
    println!("{report}Artifacts: {}", retained.display());
    Ok(())
}

fn accepted(code: Option<i32>, metric: &str, pixels: u32) -> Result<bool, String> {
    if !matches!(code, Some(0 | 1)) {
        return Err(format!("ImageMagick comparison failed: {metric}"));
    }
    let changed: f64 = metric
        .trim()
        .parse()
        .map_err(|_| format!("invalid AE metric: {metric}"))?;
    if !changed.is_finite() || changed < 0.0 || changed > f64::from(pixels) || pixels == 0 {
        return Err(format!("invalid changed-pixel count: {metric}"));
    }
    Ok(changed / f64::from(pixels) <= MAX_CHANGED_FRACTION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_changes_and_tool_failures_fail_closed() {
        assert_eq!(accepted(Some(0), "0", 10000), Ok(true));
        assert_eq!(accepted(Some(1), "5", 10000), Ok(true));
        assert_eq!(accepted(Some(1), "6", 10000), Ok(false));
        for (code, metric) in [
            (Some(2), "0"),
            (None, "0"),
            (Some(1), "NaN"),
            (Some(1), "-1"),
            (Some(1), "error"),
        ] {
            assert!(accepted(code, metric, 10000).is_err());
        }
    }

    #[test]
    fn golden_registry_covers_both_themes_and_empty_states() {
        let spec = crate::screenshots::golden_spec();
        for name in [
            "welcome-light",
            "welcome",
            "editor-dark",
            "editor-light",
            "settings-general-dark",
            "settings-general-light",
            "panels-comments-empty-dark",
            "panels-comments-empty-light",
            "panels-palette-dark",
            "panels-palette-light",
            "dialogs-about-dark",
            "dialogs-about-light",
        ] {
            assert!(
                spec.lines()
                    .any(|line| line.starts_with(&format!("{name}\t"))),
                "missing {name}"
            );
        }
    }
}
