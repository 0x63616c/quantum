//! Release preparation and separately authorized tagging. Planning is offline and read-only.

use crate::{
    Error, Result,
    cmd::{Command, Runner},
    version,
    workspace::Workspace,
};

const REPO: &str = "0x63616c/quantum";

fn command(workspace: &Workspace, program: &str, args: &[&str]) -> Result<Command> {
    let sh = xshell::Shell::new().map_err(|e| Error::Invalid(e.to_string()))?;
    Ok(Command::from(xshell::cmd!(sh, "{program} {args...}")).current_dir(workspace.root()))
}

fn git(workspace: &Workspace, runner: &dyn Runner, args: &[&str]) -> Result<String> {
    Ok(command(workspace, "git", args)?
        .allow_in_dry_run()
        .read(runner)?)
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error::Invalid(message.into()))
    }
}

/// Group NUL-separated subject/body records from `git log --format=%s%x00%b%x00`.
/// Merge commits are excluded by the caller. Other subjects are retained in Other.
pub fn changelog(log: &str, version: &str) -> String {
    #[expect(
        clippy::expect_used,
        reason = "internal invariant: constant pattern, exercised by the changelog tests"
    )]
    let conventional = regex::Regex::new(r"^([a-z]+)(?:\([^()]+\))?(!)?: (.+)$")
        .expect("constant Conventional Commit pattern");
    let mut groups = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut breaking = Vec::new();
    let parts: Vec<_> = log.split('\0').collect();
    for record in parts.chunks(2) {
        let subject = record[0].trim();
        if subject.is_empty() {
            continue;
        }
        let body = record.get(1).copied().unwrap_or_default();
        let parsed = conventional.captures(subject);
        let kind = parsed
            .as_ref()
            .and_then(|m| m.get(1))
            .map_or("", |m| m.as_str());
        let description = parsed
            .as_ref()
            .and_then(|m| m.get(3))
            .map_or(subject, |m| m.as_str());
        let bang = parsed.as_ref().is_some_and(|m| m.get(2).is_some());
        let group = match kind {
            "feat" => 0,
            "fix" => 1,
            "perf" => 2,
            "refactor" => 3,
            _ => 4,
        };
        let mut item = description.to_owned();
        if let Some((text, number)) = description.rsplit_once(" (#")
            && let Some(number) = number.strip_suffix(')')
            && !number.is_empty()
            && number.bytes().all(|b| b.is_ascii_digit())
        {
            item = format!("{text} ([#{number}](https://github.com/{REPO}/pull/{number}))");
        }
        if bang
            || body
                .lines()
                .any(|l| l.starts_with("BREAKING CHANGE:") || l.starts_with("BREAKING-CHANGE:"))
        {
            let details = body
                .lines()
                .filter_map(|l| {
                    l.strip_prefix("BREAKING CHANGE:")
                        .or_else(|| l.strip_prefix("BREAKING-CHANGE:"))
                })
                .collect::<Vec<_>>()
                .join(" ");
            breaking.push(format!(
                "- {item}{}",
                if details.is_empty() {
                    String::new()
                } else {
                    format!(": {}", details.trim())
                }
            ));
        }
        groups[group].push(format!("- {item}"));
    }
    let mut out = format!("## [{version}]\n");
    if !breaking.is_empty() {
        out.push_str(&format!(
            "\n### Breaking changes\n\n{}\n",
            breaking.join("\n")
        ));
    }
    for (name, items) in ["Added", "Fixed", "Performance", "Refactored", "Other"]
        .into_iter()
        .zip(groups)
    {
        if !items.is_empty() {
            out.push_str(&format!("\n### {name}\n\n{}\n", items.join("\n")));
        }
    }
    if !out.contains("###") {
        out.push_str("\nNo changes.\n");
    }
    out
}

fn insert_changelog(old: &str, section: &str) -> String {
    let position = old
        .match_indices("## [")
        .find(|(offset, _)| {
            (*offset == 0 || old.as_bytes()[offset - 1] == b'\n')
                && !old[*offset..].starts_with("## [Unreleased]")
        })
        .map_or(old.len(), |(offset, _)| offset);
    let prefix = &old[..position];
    let separator = if prefix.ends_with("\n\n") {
        ""
    } else if prefix.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    format!(
        "{prefix}{separator}{}\n{}",
        section.trim_end(),
        &old[position..]
    )
}

fn previous_log(workspace: &Workspace, runner: &dyn Runner) -> Result<String> {
    let tags = git(
        workspace,
        runner,
        &[
            "tag",
            "--merged",
            "HEAD",
            "--list",
            "v*",
            "--sort=-version:refname",
        ],
    )?;
    let previous = tags.lines().find(|tag| {
        tag.strip_prefix('v').is_some_and(|v| {
            semver::Version::parse(v).is_ok_and(|v| v.pre.is_empty() && v.build.is_empty())
        })
    });
    let range = previous.map_or_else(|| "HEAD".into(), |tag| format!("refs/tags/{tag}..HEAD"));
    git(
        workspace,
        runner,
        &[
            "log",
            "--no-merges",
            "--no-show-signature",
            "--no-color",
            "--format=%s%x00%b%x00",
            &range,
            "--",
        ],
    )
}

fn preflight(workspace: &Workspace, runner: &dyn Runner, tag: &str) -> Result<String> {
    require(
        git(
            workspace,
            runner,
            &["status", "--porcelain", "--untracked-files=all"],
        )?
        .is_empty(),
        "release requires a clean tree",
    )?;
    require(
        git(workspace, runner, &["branch", "--show-current"])? == "main",
        "release requires main",
    )?;
    let head = git(workspace, runner, &["rev-parse", "HEAD"])?;
    let remote = git(
        workspace,
        runner,
        &["ls-remote", "--exit-code", "origin", "refs/heads/main"],
    )?;
    require(
        remote.split_whitespace().next() == Some(head.as_str()),
        "HEAD must match origin/main; update main first",
    )?;
    require(
        git(workspace, runner, &["tag", "--list", tag])?.is_empty(),
        "release tag already exists locally",
    )?;
    require(
        git(
            workspace,
            runner,
            &["ls-remote", "--tags", "origin", &format!("refs/tags/{tag}")],
        )?
        .is_empty(),
        "release tag already exists remotely",
    )?;
    let endpoint = format!("repos/{REPO}/commits/{head}/check-runs?per_page=100");
    let checks = command(workspace, "gh", &["api", &endpoint])?.read(runner)?;
    let checks: serde_json::Value =
        serde_json::from_str(&checks).map_err(|source| Error::Json {
            context: "HEAD CI check runs".into(),
            source,
        })?;
    let runs = checks["check_runs"]
        .as_array()
        .ok_or_else(|| Error::Invalid("missing CI check runs".into()))?;
    require(
        !runs.is_empty()
            && checks["total_count"].as_u64() == Some(runs.len() as u64)
            && runs.iter().all(|run| {
                run["head_sha"] == head
                    && run["status"] == "completed"
                    && run["conclusion"] == "success"
            }),
        "all HEAD CI checks must be completed and successful (at most 100)",
    )?;
    Ok(head)
}

/// Plan by default. Explicit execution prepares a local branch/commit or pushes a
/// separately verified annotated tag. Every external operation uses the supplied runner.
pub fn run(
    workspace: &Workspace,
    runner: &dyn Runner,
    request: &str,
    execute: bool,
) -> Result<String> {
    let tagging = request == "tag";
    let current = version::workspace_version(workspace)?;
    let next = if tagging {
        current.clone()
    } else {
        version::next_version(&current, request)?
    };
    require(tagging || next > current, "release version must increase")?;
    let tag = format!("v{next}");
    let branch = format!("chore/release-{tag}");
    let title = format!("chore(release): {tag}");
    let mut report = format!(
        "{} {tag}\nPreflight: clean tree; on main; HEAD equals remote origin/main; tag absent locally/remotely; all HEAD CI checks successful via gh.\n",
        if execute {
            "Execute"
        } else {
            "Dry run (offline; preflight checks deferred)"
        }
    );
    if tagging {
        report.push_str(&format!("Verify main HEAD is merged release commit '{title}', version and CHANGELOG.md match.\ngit tag -a {tag} -m 'Release {tag}' <verified HEAD>\ngit push origin refs/tags/{tag}:refs/tags/{tag}\nTag triggers Release macOS: build, sign, notarize, staple, DMG, signed appcast, GitHub release, site redeploy.\n"));
        if execute {
            let head = preflight(workspace, runner, &tag)?;
            require(
                git(workspace, runner, &["log", "-1", "--format=%s"])? == title,
                "main HEAD must be the squash-merged release commit",
            )?;
            let parents = git(
                workspace,
                runner,
                &["rev-list", "--parents", "-n", "1", "HEAD"],
            )?;
            require(
                parents.split_whitespace().count() == 2,
                "release must be squash-merged on main",
            )?;
            let changelog = workspace.read("CHANGELOG.md")?;
            require(
                changelog
                    .lines()
                    .find(|l| l.starts_with("## [") && *l != "## [Unreleased]")
                    == Some(format!("## [{next}]").as_str()),
                "latest changelog section must match release version",
            )?;
            command(
                workspace,
                "git",
                &["tag", "-a", &tag, "-m", &format!("Release {tag}"), &head],
            )?
            .run(runner)?;
            command(
                workspace,
                "git",
                &[
                    "push",
                    "origin",
                    &format!("refs/tags/{tag}:refs/tags/{tag}"),
                ],
            )?
            .run(runner)?;
        }
        return Ok(report);
    }
    if execute {
        preflight(workspace, runner, &tag)?;
    }
    let plan = version::Plan::prepare(workspace, runner, request)?;
    let section = changelog(&previous_log(workspace, runner)?, &next.to_string());
    let old = if workspace.path("CHANGELOG.md").exists() {
        workspace.read("CHANGELOG.md")?
    } else {
        "# Changelog\n\nNotable changes for each stable release.\n".into()
    };
    require(
        !old.lines().any(|l| l == format!("## [{next}]")),
        "changelog version already exists",
    )?;
    let updated = insert_changelog(&old, &section);
    let notes = crate::macos::release_notes::render(
        &workspace.read("packaging/macos/RELEASE_NOTES.md")?,
        "## Release changes",
        section.trim_end(),
    );
    let paths_display = plan.changed_paths().join(" ");
    report.push_str(&plan.diff());
    report.push_str(&format!("\nCHANGELOG.md section:\n{section}\nRelease notes preview (signed installation template; workflow selects actual notarization status):\n{notes}\ngit switch -c {branch}\nWrite version files\ncargo update --workspace --offline\nWrite CHANGELOG.md\ngit add -- {paths_display} CHANGELOG.md\ngit commit -m '{title}'\nPrint only: gh pr create --base main --head {branch} --title '{title}'\nAfter review and squash merge, on updated main: cargo xtask release tag --execute\nTag triggers Release macOS: build, sign, notarize, staple, DMG, signed appcast, GitHub release, site redeploy.\n"));
    if execute {
        command(workspace, "git", &["switch", "-c", &branch])?.run(runner)?;
        plan.apply(runner)?;
        let path = workspace.path("CHANGELOG.md");
        std::fs::write(&path, updated).map_err(|source| Error::Io { path, source })?;
        let mut paths = vec!["add", "--"];
        paths.extend(plan.changed_paths());
        paths.push("CHANGELOG.md");
        command(workspace, "git", &paths)?.run(runner)?;
        command(workspace, "git", &["commit", "-m", &title])?.run(runner)?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests;
