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

/// Key screens, in display order. Each one is rendered from a fixed fixture at a fixed size.
struct Screen {
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
                name: "welcome",
                caption: "Welcome window (no recent projects)",
                title: "Quantum",
                size: (1024, 704),
                args: "--theme dark",
            },
            Screen {
                name: "editor-dark",
                caption: "Editor, dark theme (Autumn Workshop example)",
                title: "Autumn Workshop",
                size: (1440, 900),
                args: "--example --theme dark",
            },
            Screen {
                name: "editor-light",
                caption: "Editor, light theme (Autumn Workshop example)",
                title: "Autumn Workshop",
                size: (1440, 900),
                args: "--example --theme light",
            },
            $($(
                Screen {
                    name: concat!($state, "-dark"),
                    caption: concat!($caption, " (dark)"),
                    title: $title,
                    size: ($width, $height),
                    args: concat!("--show ", $state, " --theme dark"),
                },
                Screen {
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

pub const USAGE: &str = "cargo xtask screenshots [--base REF] [--head REF] [--screen NAME ...] [--out DIR] [--publish PR] [--remote NAME]
  --base     before ref (default: merge-base of HEAD and <remote>/main)
  --head     after ref (default: the current working tree, uncommitted changes included)
  --screen   limit to exact screen names (e.g. panels-history-dark); repeat for multiple screens; default all
             welcome, editor-{dark,light}, and every --show state suffixed -dark or -light
  --out      output directory (default: target/pr-screenshots)
  --publish  push PNGs to the pr-assets branch and fill the PR's Screenshots section
  --remote   git remote for the base default and publishing (default: origin, else the only remote)
Linux only (Xvfb + Mesa lavapipe). Needs: xvfb xdotool imagemagick dbus mesa-vulkan-drivers; gh for --publish.";

struct Options {
    base: Option<String>,
    head: Option<String>,
    screens: Vec<String>,
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
    let remote = match options.remote.clone() {
        Some(remote) => remote,
        None => default_remote(&root)?,
    };
    let out = options
        .out
        .clone()
        .unwrap_or_else(|| root.join("target/pr-screenshots"));
    let screens: Vec<&Screen> = if options.screens.is_empty() {
        SCREENS.iter().collect()
    } else {
        options
            .screens
            .iter()
            .map(|name| {
                SCREENS
                    .iter()
                    .find(|s| s.name == name)
                    .ok_or_else(|| format!("unknown screen {name:?}"))
            })
            .collect::<Result<_>>()?
    };
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
        Some(pr) => {
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
            let assets = publish_assets(&root, &remote, pr, &base, &head, &screens)?;
            Urls::Remote {
                prefix: format!(
                    "https://raw.githubusercontent.com/{repo}/{assets}/pr-{pr}/{}",
                    head.id
                ),
            }
        }
        None => Urls::Local,
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
    Ok(options)
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
fn build(tree: &Path, out: &Path, label: &str) -> Result<PathBuf> {
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
        "<sub>Before <code>{}</code>{base_name} → after <code>{}</code>. {} of {} key screens changed. Linux, Mesa lavapipe, fixed window sizes; generated by <code>cargo xtask screenshots</code>.</sub>\n",
        base.id,
        head.id,
        changed.len(),
        results.len()
    );
    if changed.is_empty() {
        md.push_str("No visible change on the key screens.\n\n");
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
        let _ = writeln!(md, "| Screen | After |\n| :--- | :---: |");
        for r in &same {
            let _ = writeln!(
                md,
                "| {} | {} |",
                r.screen.caption,
                img(urls, head, r.screen, 300)
            );
        }
        md.push_str("\n</details>\n");
    }
    md
}

/// Commit the PNGs on top of the `pr-assets` branch with plumbing commands (no checkout,
/// no force push) and return the new commit, so image URLs stay pinned to it.
fn publish_assets(
    root: &Path,
    remote: &str,
    pr: u64,
    base: &Side,
    head: &Side,
    screens: &[&Screen],
) -> Result<String> {
    let mut blobs = Vec::new();
    for side in [base, head] {
        for name in screens
            .iter()
            .flat_map(|s| [s.name.to_string(), format!("{}-zoom", s.name)])
        {
            let file = side.dir.join(format!("{name}.png"));
            if file.exists() {
                let blob = git(root, &["hash-object", "-w", &file.to_string_lossy()])?;
                blobs.push((
                    format!("pr-{pr}/{}/{}/{name}.png", head.id, side.label),
                    blob,
                ));
            }
        }
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

fn update_pr_body(root: &Path, pr: u64, markdown: &str) -> Result<()> {
    let number = pr.to_string();
    let body = gh(
        root,
        &["pr", "view", &number, "--json", "body", "-q", ".body"],
    )?;
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
mod tests {
    use super::*;

    #[test]
    fn screen_names_are_unique_and_state_screens_have_matching_themes() {
        let mut names = std::collections::BTreeSet::new();
        for screen in SCREENS {
            assert!(
                names.insert(screen.name),
                "duplicate screen {}",
                screen.name
            );
            let args: Vec<_> = screen.args.split_whitespace().collect();
            if args.first() != Some(&"--show") {
                continue;
            }
            assert_eq!(args.len(), 4);
            let state = args[1];
            assert_eq!(args[2], "--theme");
            for theme in ["dark", "light"] {
                let name = format!("{state}-{theme}");
                let partner = SCREENS
                    .iter()
                    .find(|candidate| candidate.name == name)
                    .expect("missing theme");
                assert_eq!(partner.args, format!("--show {state} --theme {theme}"));
                assert_eq!(partner.size, screen.size);
                assert_eq!(partner.title, screen.title);
            }
            assert_eq!(
                screen.size,
                if state.starts_with("settings-") {
                    (880, 640)
                } else {
                    (1440, 900)
                }
            );
        }
    }

    #[test]
    fn established_screens_keep_their_original_capture_contract() {
        let expected = [
            ("welcome", "Quantum", (1024, 704), "--theme dark"),
            (
                "editor-dark",
                "Autumn Workshop",
                (1440, 900),
                "--example --theme dark",
            ),
            (
                "editor-light",
                "Autumn Workshop",
                (1440, 900),
                "--example --theme light",
            ),
        ];
        for (screen, expected) in SCREENS.iter().zip(expected) {
            assert_eq!(
                (screen.name, screen.title, screen.size, screen.args),
                expected
            );
        }
    }

    #[test]
    fn splice_replaces_marked_block() {
        let body =
            format!("Closes #1\n\n## Screenshots\n\n{START}\nold\n{END}\n\n## Testing\n\nok\n");
        let new = splice_body(&body, "new\n");
        assert!(new.contains(&format!("{START}\nnew\n{END}")));
        assert!(!new.contains("old"));
        assert_eq!(new.matches("## Screenshots").count(), 1);
    }

    #[test]
    fn splice_adds_section_before_testing() {
        let new = splice_body(
            "Closes #1\n\n## Changes\n\n- x\n\n## Testing\n\nok\n",
            "shots\n",
        );
        let shots = new.find("## Screenshots").unwrap();
        assert!(new.find("## Changes").unwrap() < shots);
        assert!(shots < new.find("## Testing").unwrap());
    }

    #[test]
    fn splice_appends_without_testing_heading() {
        let new = splice_body("Closes #1\r\n", "shots\n");
        assert!(new.starts_with("Closes #1\n\n## Screenshots"));
        assert!(new.trim_end().ends_with(END));
    }

    #[test]
    fn parse_publish_accepts_hash() {
        let options = parse(&[
            "--publish".into(),
            "#42".into(),
            "--screen".into(),
            "welcome".into(),
        ])
        .unwrap();
        assert_eq!(options.publish, Some(42));
        assert_eq!(options.screens, ["welcome"]);
    }
}
