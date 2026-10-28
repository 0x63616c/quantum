//! Local documentation links and durable image assets. Remote URLs are not fetched.

use std::path::Path;

use crate::{
    Error, Result,
    check::{Context, Outcome},
    cmd::{Command, Runner},
    workspace::Workspace,
};

pub(crate) fn links(context: &Context<'_>) -> Outcome {
    super::outcome(check_links(context.workspace))
}

pub(crate) fn assets(context: &Context<'_>) -> Outcome {
    super::outcome(check_assets(context.workspace, context.runner))
}

/// Preserve relative URL paths before filesystem resolution, including percent
/// escapes. Schemes and authorities are external; fragments and queries are not paths.
fn local_path(target: &str) -> Option<String> {
    let path = target.split(['?', '#']).next().unwrap_or_default();
    let has_scheme = path.split_once(':').is_some_and(|(scheme, _)| {
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+-.".contains(&b))
    });
    if has_scheme {
        return None;
    }
    let path = if let Some(authority) = path.strip_prefix("//") {
        // An empty authority is local: ///file has path /file.
        if !authority.is_empty() && !authority.starts_with('/') {
            return None;
        }
        authority
    } else {
        path
    };
    Some(
        percent_encoding::percent_decode_str(path)
            .decode_utf8_lossy()
            .into_owned(),
    )
}

fn check_links(workspace: &Workspace) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    for relative in ["docs/glossary.md", "docs/adr/0003-product-direction.md"] {
        let document = workspace.path(relative);
        let targets = super::captures(&workspace.read(relative)?, r"\[[^\]]+\]\(([^\s)]+)\)")?;
        if targets.is_empty() {
            errors.push(format!("{relative}: expected at least one Markdown link"));
        }
        for target in targets {
            if let Some(path) = local_path(&target)
                && !path.is_empty()
                && !document
                    .parent()
                    .unwrap_or(workspace.root())
                    .join(path)
                    .exists()
            {
                errors.push(format!("{relative}: missing local link target: {target}"));
            }
        }
    }
    Ok(errors)
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            ["png", "jpg", "jpeg", "gif", "svg", "webp"]
                .contains(&extension.to_ascii_lowercase().as_str())
        })
}

fn image_errors(
    workspace: &Workspace,
    document: &Path,
    source: &str,
) -> Result<(usize, Vec<String>)> {
    let mut checked = 0;
    let mut errors = Vec::new();
    // This deliberately includes ordinary links to images, not just ![images].
    for target in super::captures(source, r"\[[^\]\n]*\]\(([^\s)]+)\)")? {
        let Some(path) = local_path(&target) else {
            continue;
        };
        let path = document.parent().unwrap_or(workspace.root()).join(path);
        if !is_image(&path) {
            continue;
        }
        checked += 1;
        if !path.is_file() {
            errors.push(format!("Missing documentation image: {}", path.display()));
        } else {
            let resolved = path.canonicalize().map_err(|source| Error::Io {
                path: path.clone(),
                source,
            })?;
            if resolved.starts_with(workspace.path("screenshots")) {
                errors.push(format!(
                    "{}: documentation image points into screenshots: {target}",
                    document.display()
                ));
            }
        }
    }
    Ok((checked, errors))
}

fn check_assets(workspace: &Workspace, runner: &dyn Runner) -> Result<Vec<String>> {
    let mut documents: Vec<_> = [
        "README.md",
        "AGENTS.md",
        "examples/README.md",
        "site/README.md",
        "design/ART-CHECKLIST.md",
    ]
    .iter()
    .map(|path| workspace.path(path))
    .collect();
    documents.extend(super::descendants(workspace, "docs", "*.md")?);
    documents.extend(super::descendants(workspace, "crates", "README.md")?);
    let mut checked = 0;
    let mut errors = Vec::new();
    for document in documents {
        let (count, problems) = image_errors(workspace, &document, &workspace.read(&document)?)?;
        checked += count;
        errors.extend(problems);
    }
    if checked == 0 {
        errors.push("expected at least one local documentation image".into());
    }
    errors.extend(check_ignored_assets(workspace, runner)?);
    Ok(errors)
}

fn check_ignored_assets(workspace: &Workspace, runner: &dyn Runner) -> Result<Vec<String>> {
    let captures = [
        "screenshots/app-dark.png",
        "screenshots/site/final/verification.json",
        "screenshots/review-1/tools.png",
    ];
    let images = super::paths(workspace, "docs/images/*.png")?;
    if images.is_empty() {
        return Ok(vec!["expected at least one PNG in docs/images".into()]);
    }
    // A short argument vector has the same check-ignore semantics as --stdin;
    // keep all execution in the shared runner without adding piped stdin support.
    let command = Command::new("git")
        .args(["check-ignore", "--no-index", "--"])
        .args(captures)
        .args(
            images
                .iter()
                .map(|path| path.strip_prefix(workspace.root()).unwrap_or(path)),
        )
        .current_dir(workspace.root())
        .allow_in_dry_run();
    let output = command.read(runner)?;
    let ignored: Vec<_> = output.lines().collect();
    if ignored == captures {
        Ok(Vec::new())
    } else {
        Ok(vec![format!(
            "git check-ignore: expected only capture paths {captures:?}, found {ignored:?}"
        )])
    }
}

#[cfg(test)]
mod tests;
