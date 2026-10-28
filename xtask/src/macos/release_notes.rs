//! Installation guidance plus the tagged changelog, with a commit-subject fallback.
use super::{Context, invalid, mkdir, read, stable_tag, write};
use crate::Result;

/// Combine installation guidance with a release's changes without writing files.
pub fn render(base: &str, heading: &str, changes: &str) -> String {
    format!("{}\n\n{heading}\n\n{changes}\n", base.trim_end())
}

/// Write dist/RELEASE_NOTES.md for TAG using fully qualified tag refs, strict
/// ancestors and non-merge subjects. The marker selects the installation template.
pub fn generate(context: &Context<'_>) -> Result<()> {
    let tag = context.environment.get("TAG");
    if !stable_tag(tag) {
        return Err(invalid("TAG must be a stable release version"));
    }
    let sh = xshell::Shell::new().map_err(|error| invalid(&error.to_string()))?;
    let release = format!("refs/tags/{tag}");
    let commit = format!("{release}^{{commit}}");
    context
        .script(xshell::cmd!(sh, "git rev-parse --verify {commit}"))
        .read(context.runner)?;
    let tags = context
        .script(xshell::cmd!(
            sh,
            "git tag --merged {release} --no-contains {release}"
        ))
        .read(context.runner)?;
    let matches: Vec<_> = tags
        .lines()
        .filter(|tag| stable_tag(tag))
        .map(|tag| format!("--match={tag}"))
        .collect();
    let previous = if matches.is_empty() {
        None
    } else {
        Some(
            context
                .script(xshell::cmd!(
                    sh,
                    "git describe --tags --abbrev=0 {matches...} {release}"
                ))
                .read(context.runner)?,
        )
    };
    let range = previous.as_ref().map_or_else(
        || release.clone(),
        |tag| format!("refs/tags/{tag}..{release}"),
    );
    let mut changes = context
        .script(xshell::cmd!(
            sh,
            "git log --no-merges --no-show-signature --no-color '--pretty=- %s (%h)' {range} --"
        ))
        .read(context.runner)?;
    // Read the committed tag's changelog, never an unrelated working-tree version.
    let changelog_ref = format!("{release}:CHANGELOG.md");
    let exists = context
        .script(xshell::cmd!(sh, "git cat-file -e {changelog_ref}"))
        .status(context.runner)?;
    if exists.success() {
        let changelog = context
            .script(xshell::cmd!(sh, "git show {changelog_ref}"))
            .read(context.runner)?;
        let heading = format!("## [{}]", &tag[1..]);
        if let Some(start) = changelog.lines().position(|line| line == heading) {
            changes = changelog
                .lines()
                .skip(start)
                .take_while(|line| *line == heading || !line.starts_with("## ["))
                .collect::<Vec<_>>()
                .join("\n");
        } else {
            return Err(invalid(
                "Tagged CHANGELOG.md has no matching release section",
            ));
        }
    }
    let template = if context.path("dist/.notarized").is_file() {
        "RELEASE_NOTES.md"
    } else {
        "RELEASE_NOTES.unnotarized.md"
    };
    let base = String::from_utf8(read(&context.path(format!("packaging/macos/{template}")))?)
        .map_err(|_| invalid("Release notes template is not UTF-8"))?;
    let heading = format!(
        "## Changes since {}",
        previous.as_deref().unwrap_or("the beginning")
    );
    mkdir(&context.path("dist"))?;
    write(
        &context.path("dist/RELEASE_NOTES.md"),
        render(&base, &heading, &changes),
    )
}
