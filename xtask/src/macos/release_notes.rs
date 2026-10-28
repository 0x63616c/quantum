//! Installation notes plus commit subjects; never include authors or commit bodies.
use super::{Context, invalid, mkdir, read, stable_tag, write};
use crate::Result;

/// Write dist/RELEASE_NOTES.md for TAG using fully qualified tag refs, strict
/// ancestors and non-merge subjects. The marker selects the installation template.
pub fn generate(context: &Context<'_>) -> Result<()> {
    let tag = context.environment.get("TAG");
    if !stable_tag(tag) {
        return Err(invalid("TAG must be a stable release version"));
    }
    let release = format!("refs/tags/{tag}");
    context
        .command("git")
        .args(["rev-parse", "--verify", &format!("{release}^{{commit}}")])
        .read(context.runner)?;
    let tags = context
        .command("git")
        .args(["tag", "--merged", &release, "--no-contains", &release])
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
                .command("git")
                .args(["describe", "--tags", "--abbrev=0"])
                .args(matches)
                .arg(&release)
                .read(context.runner)?,
        )
    };
    let range = previous.as_ref().map_or_else(
        || release.clone(),
        |tag| format!("refs/tags/{tag}..{release}"),
    );
    let changes = context
        .command("git")
        .args([
            "log",
            "--no-merges",
            "--no-show-signature",
            "--no-color",
            "--pretty=- %s (%h)",
            &range,
            "--",
        ])
        .read(context.runner)?;
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
        format!("{}\n\n{heading}\n\n{changes}\n", base.trim_end()),
    )
}
