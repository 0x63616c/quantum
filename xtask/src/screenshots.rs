//! `cargo xtask screenshots`: before/after captures of key screens for UI pull requests.
//!
//! Builds `voxelite` at two refs, captures the same screens from each on a virtual X
//! display (`scripts/capture-pr-screens.sh`), and writes a Before | After markdown block.
//! `--publish <PR>` stores the PNGs on the `pr-assets` branch and puts that block in the
//! PR description. See `.agents/skills/ui-pr-screenshots/SKILL.md`.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

type Result<T> = std::result::Result<T, String>;

/// Capture surfaces, in display order, with fixed fixtures and sizes.
#[derive(Debug)]
struct Screen {
    key: bool,
    name: &'static str,
    caption: &'static str,
    title: &'static str,
    size: (u32, u32),
    args: &'static str,
}

// Each launch state expands into both themes, with identical geometry and title matching.
macro_rules! screens {
    ($($title:literal, ($width:literal, $height:literal) => [
        $(($state:literal, $caption:literal)),* $(,)?
    ]),* $(,)?) => {
        &[
            Screen {
                key: true,
                name: "welcome",
                caption: "Welcome window (no recent projects)",
                title: "Quantum",
                size: (1024, 704),
                args: "--theme dark",
            },
            Screen {
                key: true,
                name: "editor-dark",
                caption: "Editor, dark theme (Autumn Workshop example)",
                title: "Autumn Workshop",
                size: (1440, 900),
                args: "--example --theme dark",
            },
            Screen {
                key: true,
                name: "editor-light",
                caption: "Editor, light theme (Autumn Workshop example)",
                title: "Autumn Workshop",
                size: (1440, 900),
                args: "--example --theme light",
            },
            $($(
                Screen {
                    key: false,
                    name: concat!($state, "-dark"),
                    caption: concat!($caption, " (dark)"),
                    title: $title,
                    size: ($width, $height),
                    args: concat!("--show ", $state, " --theme dark"),
                },
                Screen {
                    key: false,
                    name: concat!($state, "-light"),
                    caption: concat!($caption, " (light)"),
                    title: $title,
                    size: ($width, $height),
                    args: concat!("--show ", $state, " --theme light"),
                },
            )*)*
        ]
    };
}

const SCREENS: &[Screen] = screens![
    "Autumn Workshop", (1440, 900) => [
        ("chrome-menu-file", "File menu"),
        ("chrome-menu-edit", "Edit menu"),
        ("chrome-menu-view", "View menu"),
        ("chrome-menu-tools", "Tools menu"),
        ("chrome-menu-palette", "Palette menu"),
        ("chrome-menu-layers", "Layers menu"),
        ("chrome-menu-agent", "Agent menu"),
        ("chrome-menu-extensions", "Extensions menu"),
        ("chrome-menu-window", "Window menu"),
        ("chrome-menu-help", "Help menu"),
        ("panels-palette", "Focused palette sidebar"),
        ("panels-history", "History and named versions"),
        ("panels-history-preview", "History preview and restore controls"),
        ("panels-comments-empty", "Empty resolved comments"),
        ("panels-comments", "Comments and model pins"),
        ("panels-comment-thread", "Comment thread"),
        ("panels-comment-draft", "New comment composer"),
        ("panels-agent", "Expanded agent activity"),
        ("panels-agent-inspection", "Agent inspection thumbnails"),
        ("popovers-connect-agent", "Connect agent"),
        ("dialogs-about", "About Quantum"),
        ("dialogs-acknowledgements", "Acknowledgements"),
        ("dialogs-keyboard-guide", "Keyboard guide"),
        ("dialogs-commands", "Command search"),
        ("dialogs-tools", "Tool search"),
        ("dialogs-extensions", "Extensions"),
        ("dialogs-unsaved-changes", "Unsaved changes"),
        ("dialogs-save-waiting", "Save dialog waiting surface"),
        ("popovers-palette-chooser", "Palette chooser"),
        ("dialogs-palettes", "Palette manager and color editor"),
        ("popovers-palette-menu", "Palette import/export menu"),
        ("dialogs-palette-delete", "Delete custom palette confirmation"),
        ("overlays-debug", "Debug overlay"),
    ],
    "^Quantum Settings$", (880, 640) => [
        ("settings-general", "Settings: General"),
        ("settings-appearance", "Settings: Appearance"),
        ("settings-viewport", "Settings: Viewport / Rendering"),
        ("settings-shortcuts", "Settings: Shortcuts"),
        ("settings-agent", "Settings: Agent"),
        ("settings-updates", "Settings: Updates"),
        ("settings-advanced", "Settings: Advanced"),
    ],
];

const ASSETS_BRANCH: &str = "pr-assets";
const START: &str = "<!-- screenshots:start -->";
const END: &str = "<!-- screenshots:end -->";
/// Image width in the PR table; two columns fit GitHub's conversation view.
const CELL_WIDTH: u32 = 440;
/// Fraction of pixels allowed to differ (after a 2% colour fuzz) before a screen counts as changed.
const CHANGED_FRACTION: f64 = 0.0005;

pub const USAGE: &str = "cargo xtask screenshots [--base REF] [--head REF] [--screen SELECTOR ...] [--pr-body FILE] [--out DIR] [--publish PR] [--remote NAME]
  --base     before ref (default: merge-base of HEAD and <remote>/main)
  --head     after ref (default: the current working tree, uncommitted changes included)
  --screen   exact name or trailing * prefix (quote patterns); repeat to combine, '*' selects all
             default: welcome, editor-dark, editor-light; other surfaces are opt-in
  --pr-body  add selectors from one 'Screens: panels-* settings-general-*' line in FILE to the key set
             --publish reads the PR body automatically unless --screen or --pr-body is supplied
  --out      output directory (default: target/pr-screenshots)
  --publish  push PNGs to the pr-assets branch and fill the PR's Screenshots section
  --remote   git remote for the base default and publishing (default: origin, else the only remote)
Linux only (Xvfb + Mesa lavapipe). Needs: xvfb xdotool imagemagick dbus mesa-vulkan-drivers; gh for --publish.";

struct Options {
    base: Option<String>,
    head: Option<String>,
    screens: Vec<String>,
    pr_body: Option<PathBuf>,
    out: Option<PathBuf>,
    publish: Option<u64>,
    remote: Option<String>,
}

struct Side {
    label: &'static str,
    /// Short commit id, with `+dirty` for an uncommitted working tree.
    id: String,
    dir: PathBuf,
}

pub fn run(args: &[String]) -> Result<()> {
    let options = parse(args)?;
    if !cfg!(target_os = "linux") {
        return Err("cargo xtask screenshots captures on Linux (Xvfb + lavapipe) for reproducible pixels; run it on a Linux box or let the PR screenshots CI job do it".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("no workspace root")?
        .to_path_buf();
    let body = match (&options.pr_body, options.publish) {
        (Some(file), _) => {
            Some(fs::read_to_string(file).map_err(|e| format!("read {}: {e}", file.display()))?)
        }
        (None, Some(pr)) if options.screens.is_empty() => Some(pr_body(&root, pr)?),
        _ => None,
    };
    // Validate untrusted selectors before cleaning output, building or publishing.
    let screens = selected_screens(&options.screens, body.as_deref())?;
    let remote = match options.remote.clone() {
        Some(remote) => remote,
        None => default_remote(&root)?,
    };
    let out = options
        .out
        .clone()
        .unwrap_or_else(|| root.join("target/pr-screenshots"));
    eprintln!(
        "selected {} screens: {}",
        screens.len(),
        screens
            .iter()
            .map(|s| s.name)
            .collect::<Vec<_>>()
            .join(", ")
    );
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| format!("clean {}: {e}", out.display()))?;
    }
    fs::create_dir_all(out.join("bin")).map_err(|e| e.to_string())?;
    let spec = out.join("screens.tsv");
    let mut spec_text = String::new();
    for s in &screens {
        let _ = writeln!(
            spec_text,
            "{}\t{}\t{}\t{}\t{}",
            s.name, s.title, s.size.0, s.size.1, s.args
        );
    }
    fs::write(&spec, spec_text).map_err(|e| e.to_string())?;

    let base_ref = match options.base.clone() {
        Some(base) => base,
        None => git(&root, &["merge-base", "HEAD", &format!("{remote}/main")])?,
    };
    let base_sha = commit(&root, &base_ref)?;
    let base = Side {
        label: "before",
        id: short(&base_sha),
        dir: out.join("before"),
    };
    let base_bin = build_at_commit(&root, &out, &base_sha, "before")?;
    let (head, head_bin) = match &options.head {
        Some(head_ref) => {
            let sha = commit(&root, head_ref)?;
            let bin = build_at_commit(&root, &out, &sha, "after")?;
            (
                Side {
                    label: "after",
                    id: short(&sha),
                    dir: out.join("after"),
                },
                bin,
            )
        }
        None => {
            let sha = commit(&root, "HEAD")?;
            let dirty = !git(&root, &["status", "--porcelain", "--untracked-files=no"])?.is_empty();
            let bin = build(&root, &out, "after")?;
            let id = if dirty {
                format!("{}+dirty", short(&sha))
            } else {
                short(&sha)
            };
            (
                Side {
                    label: "after",
                    id,
                    dir: out.join("after"),
                },
                bin,
            )
        }
    };
    let script = root.join("scripts/capture-pr-screens.sh");
    for (side, bin) in [(&base, &base_bin), (&head, &head_bin)] {
        eprintln!("capturing {} ({})", side.label, side.id);
        status(Command::new(&script).arg(bin).arg(&side.dir).arg(&spec))?;
    }
    let results: Vec<Comparison> = screens
        .iter()
        .map(|s| compare(s, &base.dir, &head.dir))
        .collect();

    let urls = match options.publish {
        Some(pr) if results.iter().any(|r| r.state.has_assets()) => {
            let repo = gh(
                &root,
                &[
                    "repo",
                    "view",
                    "--json",
                    "nameWithOwner",
                    "-q",
                    ".nameWithOwner",
                ],
            )?;
            let assets = publish_assets(&root, &remote, pr, &base, &head, &results)?;
            Urls::Remote {
                prefix: format!(
                    "https://raw.githubusercontent.com/{repo}/{assets}/pr-{pr}/{}",
                    head.id
                ),
            }
        }
        _ => Urls::Local,
    };
    let markdown = render(&results, &base, &head, &base_ref, &urls);
    fs::write(out.join("screenshots.md"), &markdown).map_err(|e| e.to_string())?;
    if let Some(pr) = options.publish {
        update_pr_body(&root, pr, &markdown)?;
        eprintln!("updated the Screenshots section of PR #{pr}");
    }
    for r in &results {
        eprintln!("{:>13}  {}", r.state.word(), r.screen.name);
    }
    eprintln!("wrote {}", out.join("screenshots.md").display());
    Ok(())
}

fn parse(args: &[String]) -> Result<Options> {
    let mut options = Options {
        base: None,
        head: None,
        screens: Vec::new(),
        pr_body: None,
        out: None,
        publish: None,
        remote: None,
    };
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .cloned()
                .ok_or_else(|| format!("{arg} needs a value\n{USAGE}"))
        };
        match arg.as_str() {
            "--base" => options.base = Some(value()?),
            "--head" => options.head = Some(value()?),
            "--screen" => options.screens.push(value()?),
            "--pr-body" => options.pr_body = Some(value()?.into()),
            "--out" => options.out = Some(value()?.into()),
            "--remote" => options.remote = Some(value()?),
            "--publish" => {
                let pr = value()?;
                options.publish = Some(
                    pr.trim_start_matches('#')
                        .parse()
                        .map_err(|_| format!("--publish takes a PR number, not {pr:?}"))?,
                );
            }
            _ => return Err(format!("unknown argument {arg:?}\n{USAGE}")),
        }
    }
    if options.pr_body.is_some() && !options.screens.is_empty() {
        return Err("--screen and --pr-body cannot be combined".into());
    }
    Ok(options)
}

/// Resolve selectors to registry entries, deduplicated in registry order. Only a
/// trailing `*` is supported; no shell expansion or user-provided app args exist.
fn selected_screens(selectors: &[String], body: Option<&str>) -> Result<Vec<&'static Screen>> {
    let extras = body.map(body_selectors).transpose()?.unwrap_or_default();
    let mut selected: Vec<bool> = SCREENS
        .iter()
        .map(|s| selectors.is_empty() && s.key)
        .collect();
    for selector in selectors.iter().chain(&extras) {
        let prefix = selector.strip_suffix('*');
        let stem = prefix.unwrap_or(selector);
        if selector.is_empty()
            || !stem
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(format!(
                "invalid screen selector {selector:?}: use an exact name or trailing * prefix"
            ));
        }
        let mut matched = false;
        for (screen, selected) in SCREENS.iter().zip(&mut selected) {
            if prefix.map_or(screen.name == selector, |prefix| {
                screen.name.starts_with(prefix)
            }) {
                *selected = true;
                matched = true;
            }
        }
        if !matched {
            return Err(format!(
                "unknown screen selector {selector:?}: matched no screens"
            ));
        }
    }
    Ok(SCREENS
        .iter()
        .zip(selected)
        .filter_map(|(s, selected)| selected.then_some(s))
        .collect())
}

/// A deliberately small line format, not Markdown or shell syntax. The directive
/// must start at column zero; indented examples and inline mentions are ignored.
fn body_selectors(body: &str) -> Result<Vec<String>> {
    let mut lines = body
        .lines()
        .filter_map(|line| line.strip_prefix("Screens:"));
    let selectors = lines.next().unwrap_or_default();
    if lines.next().is_some() {
        return Err("use only one Screens: line in the PR body".into());
    }
    Ok(selectors.split_whitespace().map(str::to_string).collect())
}

fn build_at_commit(root: &Path, out: &Path, sha: &str, label: &str) -> Result<PathBuf> {
    let tree = out.join(format!("src-{label}"));
    git(
        root,
        &[
            "worktree",
            "add",
            "--detach",
            "--force",
            &tree.to_string_lossy(),
            sha,
        ],
    )?;
    let built = build(&tree, out, label);
    let _ = git(
        root,
        &["worktree", "remove", "--force", &tree.to_string_lossy()],
    );
    built
}

/// Debug build, sharing the workspace target directory so dependencies stay cached.
pub(crate) fn build(tree: &Path, out: &Path, label: &str) -> Result<PathBuf> {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../target"));
    eprintln!("building voxelite ({label}) in {}", tree.display());
    status(
        Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .current_dir(tree)
            .env("CARGO_TARGET_DIR", &target)
            .args([
                "build",
                "--locked",
                "-p",
                "voxelite-app",
                "--bin",
                "voxelite",
            ]),
    )?;
    let bin = out.join("bin").join(label);
    fs::copy(target.join("debug/voxelite"), &bin).map_err(|e| format!("copy binary: {e}"))?;
    Ok(bin)
}

#[derive(PartialEq)]
enum State {
    Changed,
    Unchanged,
    Added,
    Removed,
    Missing,
}

impl State {
    fn has_assets(&self) -> bool {
        matches!(self, Self::Changed | Self::Added | Self::Removed)
    }

    fn word(&self) -> &'static str {
        match self {
            State::Changed => "changed",
            State::Unchanged => "unchanged",
            State::Added => "new",
            State::Removed => "gone",
            State::Missing => "not captured",
        }
    }
}

struct Comparison<'a> {
    screen: &'a Screen,
    state: State,
    /// Integer scale of the `<name>-zoom.png` crops of the changed area, when one was made.
    zoom: Option<u32>,
}

fn compare<'a>(screen: &'a Screen, before: &Path, after: &Path) -> Comparison<'a> {
    let file = format!("{}.png", screen.name);
    let (a, b) = (before.join(&file), after.join(&file));
    let state = match (a.exists(), b.exists()) {
        (false, false) => State::Missing,
        (false, true) => State::Added,
        (true, false) => State::Removed,
        (true, true) => {
            let output = Command::new("compare")
                .args(["-metric", "AE", "-fuzz", "2%"])
                .arg(&a)
                .arg(&b)
                .arg("null:")
                .stderr(Stdio::piped())
                .output();
            let pixels = f64::from(screen.size.0 * screen.size.1);
            match output.ok().and_then(|o| {
                String::from_utf8_lossy(&o.stderr)
                    .split_whitespace()
                    .next()?
                    .parse::<f64>()
                    .ok()
            }) {
                Some(diff) if diff / pixels <= CHANGED_FRACTION => State::Unchanged,
                // Different sizes or any other compare failure counts as a change.
                _ => State::Changed,
            }
        }
    };
    let zoom = if state == State::Changed {
        zoom_crops(
            screen,
            &a,
            &b,
            &before.join(format!("{}-zoom.png", screen.name)),
            &after.join(format!("{}-zoom.png", screen.name)),
        )
    } else {
        None
    };
    Comparison {
        screen,
        state,
        zoom,
    }
}

/// Rendered width the zoomed crop should fill (two of them side by side in the PR).
const ZOOM_TARGET_WIDTH: u32 = 880;

/// When the difference is confined to a small region, write matching crops of it from both
/// sides, enlarged with nearest-neighbour scaling so small shifts are visible on the PR page.
fn zoom_crops(screen: &Screen, a: &Path, b: &Path, out_a: &Path, out_b: &Path) -> Option<u32> {
    // White where pixels differ (beyond the fuzz), black elsewhere; then its bounding box.
    let mask = Command::new("sh")
        .arg("-c")
        .arg(r#"compare -fuzz 2% "$1" "$2" -compose src -highlight-color white -lowlight-color black png:- | convert png:- -bordercolor black -border 1 -trim -format '%w %h %X %Y' info:"#)
        .arg("sh")
        .arg(a)
        .arg(b)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&mask.stdout);
    let nums: Vec<i64> = text
        .split_whitespace()
        .filter_map(|n| n.trim_start_matches('+').parse().ok())
        .collect();
    let [w, h, x, y] = nums[..] else { return None };
    let (img_w, img_h) = (i64::from(screen.size.0), i64::from(screen.size.1));
    // Undo the 1px border, pad generously for context, keep a readable minimum size.
    let (x, y) = (x - 1, y - 1);
    let pad = 48;
    let cw = (w + 2 * pad).max(360).min(img_w);
    let ch = (h + 2 * pad).max(160).min(img_h);
    if cw * ch * 4 > img_w * img_h {
        return None; // A large change reads fine in the full screenshots.
    }
    let cx = (x + w / 2 - cw / 2).clamp(0, img_w - cw);
    let cy = (y + h / 2 - ch / 2).clamp(0, img_h - ch);
    let scale = (i64::from(ZOOM_TARGET_WIDTH) / cw).clamp(1, 3) as u32;
    for (src, dst) in [(a, out_a), (b, out_b)] {
        let ok = Command::new("convert")
            .arg(src)
            .args(["-crop", &format!("{cw}x{ch}+{cx}+{cy}"), "+repage"])
            .args(["-filter", "point", "-resize", &format!("{}%", scale * 100)])
            .arg(dst)
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            return None;
        }
    }
    Some(scale)
}

enum Urls {
    Local,
    Remote { prefix: String },
}

impl Urls {
    fn image(&self, side: &Side, file: &str) -> String {
        match self {
            Urls::Local => format!("{}/{file}.png", side.label),
            Urls::Remote { prefix } => format!("{prefix}/{}/{file}.png", side.label),
        }
    }
}

fn img(urls: &Urls, side: &Side, screen: &Screen, width: u32) -> String {
    let url = urls.image(side, screen.name);
    format!(
        r#"<a href="{url}"><img src="{url}" width="{width}" alt="{} — {}"></a>"#,
        screen.caption, side.label
    )
}

fn zoom_img(urls: &Urls, side: &Side, screen: &Screen) -> String {
    let url = urls.image(side, &format!("{}-zoom", screen.name));
    format!(
        r#"<a href="{url}"><img src="{url}" width="{CELL_WIDTH}" alt="{} — {}, zoomed to the change"></a>"#,
        screen.caption, side.label
    )
}

fn render(results: &[Comparison], base: &Side, head: &Side, base_ref: &str, urls: &Urls) -> String {
    let mut md = String::new();
    let changed: Vec<_> = results
        .iter()
        .filter(|r| r.state != State::Unchanged)
        .collect();
    let same: Vec<_> = results
        .iter()
        .filter(|r| r.state == State::Unchanged)
        .collect();
    let base_name = if base_ref.len() == 40 {
        String::new()
    } else {
        format!(" ({base_ref})")
    };
    let _ = writeln!(
        md,
        "<sub>Before <code>{}</code>{base_name} → after <code>{}</code>. {} of {} selected screens changed or unavailable. Linux, Mesa lavapipe, fixed window sizes; generated by <code>cargo xtask screenshots</code>.</sub>\n",
        base.id,
        head.id,
        changed.len(),
        results.len()
    );
    if changed.is_empty() {
        md.push_str("No visible change on the selected screens.\n\n");
    }
    for r in &changed {
        let s = r.screen;
        let _ = writeln!(md, "#### {}\n", s.caption);
        let _ = writeln!(md, "| Before | After |\n| :---: | :---: |");
        let cell = |side: &Side, present: bool| {
            if present {
                img(urls, side, s, CELL_WIDTH)
            } else {
                "<em>not on this side</em>".to_string()
            }
        };
        let (has_before, has_after) = match r.state {
            State::Added => (false, true),
            State::Removed => (true, false),
            State::Missing => (false, false),
            _ => (true, true),
        };
        let _ = writeln!(
            md,
            "| {} | {} |",
            cell(base, has_before),
            cell(head, has_after)
        );
        if let Some(scale) = r.zoom {
            let _ = writeln!(
                md,
                "| {}<br><sub>Changed area, {scale}× zoom</sub> | {}<br><sub>Changed area, {scale}× zoom</sub> |",
                zoom_img(urls, base, s),
                zoom_img(urls, head, s)
            );
        }
        let _ = writeln!(
            md,
            "\n<sub><code>voxelite {}</code> at {}×{}</sub>\n",
            s.args, s.size.0, s.size.1
        );
    }
    if !same.is_empty() {
        let _ = writeln!(
            md,
            "<details><summary>Unchanged screens ({})</summary>\n",
            same.len()
        );
        for r in &same {
            let _ = writeln!(md, "- {} (`{}`)", r.screen.caption, r.screen.name);
        }
        md.push_str("\n</details>\n");
    }
    md
}

/// Publish only images referenced by changed/new/removed rows, including useful
/// zooms. Unchanged captures remain local or in CI artifacts, never in Git.
fn asset_files(results: &[Comparison], base: &Side, head: &Side) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    for r in results.iter().filter(|r| r.state.has_assets()) {
        for side in [base, head] {
            if (r.state == State::Added && side.label == base.label)
                || (r.state == State::Removed && side.label == head.label)
            {
                continue;
            }
            let name = r.screen.name;
            files.push((
                side.dir.join(format!("{name}.png")),
                format!("{}/{name}.png", side.label),
            ));
            if r.zoom.is_some() {
                files.push((
                    side.dir.join(format!("{name}-zoom.png")),
                    format!("{}/{name}-zoom.png", side.label),
                ));
            }
        }
    }
    files
}

/// Commit the PNGs on top of the `pr-assets` branch with plumbing commands (no checkout,
/// no force push) and return the new commit, so image URLs stay pinned to it.
fn publish_assets(
    root: &Path,
    remote: &str,
    pr: u64,
    base: &Side,
    head: &Side,
    results: &[Comparison],
) -> Result<String> {
    let mut blobs = Vec::new();
    for (file, path) in asset_files(results, base, head) {
        let blob = git(root, &["hash-object", "-w", &file.to_string_lossy()])?;
        blobs.push((format!("pr-{pr}/{}/{path}", head.id), blob));
    }
    let index = std::env::temp_dir().join(format!("pr-assets-index-{}", std::process::id()));
    for attempt in 1..=5 {
        let _ = fs::remove_file(&index);
        let parent = git(root, &["fetch", "--quiet", remote, ASSETS_BRANCH])
            .and_then(|_| git(root, &["rev-parse", "FETCH_HEAD"]))
            .ok();
        let with_index = |args: &[&str]| -> Result<String> {
            output(
                Command::new("git")
                    .current_dir(root)
                    .env("GIT_INDEX_FILE", &index)
                    .args(args),
            )
        };
        match &parent {
            Some(parent) => with_index(&["read-tree", parent])?,
            None => with_index(&["read-tree", "--empty"])?,
        };
        for (path, blob) in &blobs {
            with_index(&[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("100644,{blob},{path}"),
            ])?;
        }
        let tree = with_index(&["write-tree"])?;
        let message = format!("PR #{pr} screenshots at {}", head.id);
        let mut commit_args = vec!["commit-tree", tree.as_str(), "-m", message.as_str()];
        if let Some(parent) = &parent {
            commit_args.extend(["-p", parent.as_str()]);
        }
        let commit = git(root, &commit_args)?;
        match git(
            root,
            &[
                "push",
                "--quiet",
                remote,
                &format!("{commit}:refs/heads/{ASSETS_BRANCH}"),
            ],
        ) {
            Ok(_) => {
                let _ = fs::remove_file(&index);
                return Ok(commit);
            }
            Err(error) if attempt < 5 => {
                eprintln!("push to {ASSETS_BRANCH} raced ({error}); retrying")
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!()
}

fn pr_body(root: &Path, pr: u64) -> Result<String> {
    let number = pr.to_string();
    gh(
        root,
        &["pr", "view", &number, "--json", "body", "-q", ".body"],
    )
}

fn update_pr_body(root: &Path, pr: u64, markdown: &str) -> Result<()> {
    let number = pr.to_string();
    let body = pr_body(root, pr)?;
    let new = splice_body(&body, markdown);
    let file = std::env::temp_dir().join(format!("pr-{pr}-body.md"));
    fs::write(&file, new).map_err(|e| e.to_string())?;
    gh(
        root,
        &[
            "pr",
            "edit",
            &number,
            "--body-file",
            &file.to_string_lossy(),
        ],
    )?;
    Ok(())
}

/// Replace the marked block, or add a `## Screenshots` section before `## Testing`.
fn splice_body(body: &str, markdown: &str) -> String {
    let block = format!("{START}\n{}{END}", markdown);
    let body = body.replace("\r\n", "\n");
    if let (Some(start), Some(end)) = (body.find(START), body.find(END))
        && start < end
    {
        return format!("{}{block}{}", &body[..start], &body[end + END.len()..]);
    }
    let section = format!("## Screenshots\n\n{block}\n\n");
    match body.find("\n## Testing") {
        Some(at) => format!("{}\n{section}{}", &body[..at], &body[at + 1..]),
        None => format!("{}\n\n{section}", body.trim_end()),
    }
}

fn default_remote(root: &Path) -> Result<String> {
    let remotes = git(root, &["remote"])?;
    let remotes: Vec<&str> = remotes.lines().collect();
    if remotes.contains(&"origin") {
        return Ok("origin".into());
    }
    match remotes.as_slice() {
        [only] => Ok((*only).to_string()),
        _ => Err("cannot pick a git remote; pass --remote".into()),
    }
}

fn commit(root: &Path, reference: &str) -> Result<String> {
    git(
        root,
        &["rev-parse", "--verify", &format!("{reference}^{{commit}}")],
    )
}

fn short(sha: &str) -> String {
    sha.chars().take(7).collect()
}

fn git(dir: &Path, args: &[&str]) -> Result<String> {
    output(Command::new("git").current_dir(dir).args(args))
}

fn gh(dir: &Path, args: &[&str]) -> Result<String> {
    output(Command::new("gh").current_dir(dir).args(args))
}

fn output(command: &mut Command) -> Result<String> {
    let out = command.output().map_err(|e| format!("{command:?}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{command:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn status(command: &mut Command) -> Result<()> {
    let status = command.status().map_err(|e| format!("{command:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{command:?} exited with {status}"))
    }
}

#[cfg(test)]
mod tests;

/// Share the PR capture registry with the stricter native golden gate.
pub(crate) fn golden_spec() -> String {
    let mut spec = String::new();
    // Welcome's established PR key is dark-only; explicitly add its light counterpart.
    spec.push_str("welcome-light\tQuantum\t1024\t704\t--theme light\n");
    for screen in SCREENS.iter().filter(|s| {
        s.key
            || s.name.starts_with("settings-")
            || s.name.starts_with("panels-palette-")
            || s.name.starts_with("panels-comments-empty-")
            || s.name.starts_with("dialogs-about-")
    }) {
        let _ = writeln!(
            spec,
            "{}\t{}\t{}\t{}\t{}",
            screen.name, screen.title, screen.size.0, screen.size.1, screen.args
        );
    }
    spec
}
