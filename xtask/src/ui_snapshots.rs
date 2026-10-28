//! Headless GPUI golden gate, separate from PR before/after desktop captures.
mod comparison;
#[cfg(test)]
mod tests;

use crate::screenshots::{SCREENS, Screen};
use std::{collections::BTreeSet, fs, path::Path, time::Instant};
use xtask::{
    cmd::{Command, SystemRunner},
    workspace::Workspace,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const GOLDENS: &str = "crates/quantum-app/tests/ui-goldens";
const FIXTURE_CLOCK: &str = "1782048000";

#[derive(clap::Args)]
pub(super) struct Options {
    /// Explicitly replace selected goldens after all captures succeed
    #[arg(long)]
    rebaseline: bool,
    /// Exact name or trailing * prefix; repeat to combine (default: all key screens)
    #[arg(long, value_name = "NAME|PREFIX*")]
    screen: Vec<String>,
}

fn registry() -> Vec<&'static Screen> {
    SCREENS
        .iter()
        .filter(|s| {
            s.name.starts_with("welcome")
                || s.name.starts_with("editor-")
                || s.name.starts_with("settings-")
                || s.name.starts_with("panels-palette-")
                || s.name.starts_with("panels-comments-empty-")
                || s.name.starts_with("dialogs-about-")
        })
        .collect()
}

fn selected(selectors: &[String]) -> Result<Vec<&'static Screen>> {
    let all = registry();
    let mut names = BTreeSet::new();
    for selector in selectors {
        let prefix = selector.strip_suffix('*');
        let stem = prefix.unwrap_or(selector);
        if selector.is_empty()
            || !stem
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(format!(
                "invalid screen selector {selector:?}: use an exact name or trailing * prefix"
            )
            .into());
        }
        let matched: Vec<_> = all
            .iter()
            .filter(|s| prefix.map_or(s.name == selector, |p| s.name.starts_with(p)))
            .collect();
        if matched.is_empty() {
            return Err(format!("unknown screen selector {selector:?}").into());
        }
        names.extend(matched.into_iter().map(|s| s.name));
    }
    Ok(all
        .into_iter()
        .filter(|s| selectors.is_empty() || names.contains(s.name))
        .collect())
}

fn validate_files(directory: &Path, names: &BTreeSet<String>, required: bool) -> Result<()> {
    let mut found = BTreeSet::new();
    if directory.exists() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !names.contains(&name) || !entry.file_type()?.is_file() {
                return Err(format!("unknown snapshot file: {}", entry.path().display()).into());
            }
            found.insert(name);
        }
    }
    if required && &found != names {
        return Err(format!(
            "missing snapshots in {}: {:?}",
            directory.display(),
            names.difference(&found).collect::<Vec<_>>()
        )
        .into());
    }
    Ok(())
}

pub(super) fn run(options: Options) -> Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(
            "UI snapshots require Linux with Mesa lavapipe; native macOS goldens are deferred"
                .into(),
        );
    }
    let started = Instant::now();
    let screens = selected(&options.screen)?;
    let workspace = Workspace::discover()?;
    let root = workspace.root();
    let goldens = root.join(GOLDENS);
    let names = registry()
        .iter()
        .map(|s| format!("{}.png", s.name))
        .collect();
    validate_files(&goldens, &names, false)?;
    let out = root.join("target/ui-snapshots");
    fs::create_dir_all(&out)?;
    let actual = out.join("actual");
    for directory in [
        &actual,
        &out.join("expected"),
        &out.join("diff"),
        &out.join("data"),
    ] {
        if directory.exists() {
            fs::remove_dir_all(directory)?;
        }
        fs::create_dir_all(directory)?;
    }
    // All assets, history and preferences stay beneath this harness-owned directory.
    let home = out.join("home");
    fs::create_dir_all(&home)?;
    fs::write(home.join("clock.txt"), FIXTURE_CLOCK)?;
    capture(root, &out, &home, &actual, &screens)?;
    let selected_names = screens.iter().map(|s| format!("{}.png", s.name)).collect();
    validate_files(&actual, &selected_names, true)?;
    if options.rebaseline {
        fs::create_dir_all(&goldens)?;
        for name in &selected_names {
            fs::copy(actual.join(name), goldens.join(name))?;
        }
        eprintln!(
            "Rebaselined {} screens; review every PNG before committing",
            screens.len()
        );
    } else {
        compare_all(&screens, &goldens, &out)?;
        validate_files(&goldens, &names, true)?;
    }
    eprintln!(
        "UI snapshots: {} screens in {:.2}s",
        screens.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn capture(root: &Path, out: &Path, home: &Path, actual: &Path, screens: &[&Screen]) -> Result<()> {
    let runner = SystemRunner::default();
    let shell = xshell::Shell::new()?;
    let build: Command = xshell::cmd!(
        shell,
        "cargo build --locked -p quantum-app --bin quantum --features ui-snapshots"
    )
    .into();
    build.current_dir(root).run(&runner)?;
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target"));
    // Other worktrees share Cargo artifacts. Keep this exact feature build alive.
    let binary = out.join("quantum-headless");
    fs::copy(target.join("debug/quantum"), &binary)?;
    let icd = Path::new("/usr/share/vulkan/icd.d/lvp_icd.json");
    if !icd.is_file() {
        return Err("install mesa-vulkan-drivers: lavapipe ICD missing".into());
    }
    for (index, screen) in screens.iter().enumerate() {
        let name = screen.name;
        let (width, height) = (screen.size.0.to_string(), screen.size.1.to_string());
        let output = actual.join(format!("{name}.png"));
        // The app binds its session socket inside the data directory. Unix socket
        // paths are limited to ~104 bytes, so keep this per-screen path short.
        let data = out.join("data").join(index.to_string());
        fs::create_dir_all(&data)?;
        if name.starts_with("welcome-recents-") {
            xtask::screenshot_fixtures::prepare_welcome(root, home, &data)?;
        }
        let capture: Command = xshell::cmd!(
            shell,
            "{binary} --ui-snapshot {name} {width} {height} {output} {home}"
        )
        .into();
        capture
            .current_dir(root)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env("VK_ICD_FILENAMES", icd)
            .env("XDG_RUNTIME_DIR", "/tmp")
            .env("TZ", "UTC")
            .env("SEED", "0")
            .env("QUANTUM_DATA_DIR", &data)
            .env_remove("QUANTUM_CAPTURE_STATUS")
            .run(&runner)?;
        let image = comparison::Image::read(&output)?;
        if (image.width, image.height) != (screen.size.0 * 2, screen.size.1 * 2) {
            return Err(format!("{name}: wrong render dimensions").into());
        }
    }
    Ok(())
}

fn compare_all(screens: &[&Screen], goldens: &Path, out: &Path) -> Result<()> {
    let mut failures = Vec::new();
    for screen in screens {
        let file = format!("{}.png", screen.name);
        let actual = comparison::Image::read(&out.join("actual").join(&file))?;
        let expected_path = goldens.join(&file);
        if !expected_path.is_file() {
            failures.push(format!("{}: missing golden", screen.name));
            continue;
        }
        let expected = comparison::Image::read(&expected_path)?;
        let result = comparison::compare(&actual, &expected);
        if !result.passed {
            fs::copy(&expected_path, out.join("expected").join(&file))?;
            result.diff.write(&out.join("diff").join(&file))?;
            failures.push(format!(
                "{}: {} changed pixels; dimensions {:?} vs {:?}",
                screen.name,
                result.changed,
                (actual.width, actual.height),
                (expected.width, expected.height)
            ));
        }
    }
    if !failures.is_empty() {
        return Err(format!(
            "UI snapshot comparison failed:\n{}\nEvidence: {}",
            failures.join("\n"),
            out.display()
        )
        .into());
    }
    Ok(())
}
