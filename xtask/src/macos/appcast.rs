//! Signed update feeds: download history separately from signing, validate it,
//! and preserve generated XML/HTML bytes unchanged when publishing local files.
use super::{
    Context, DOWNLOAD, REPOSITORY, copy, feed,
    files::{self, WorkDir},
    invalid, mkdir, read, stable_tag, stable_version, write,
};
use crate::{Error, Result};
use serde::Deserialize;
use std::{collections::BTreeMap, fs::OpenOptions, io::Write, path::Path};

#[derive(Deserialize)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

/// Fetch the numerically highest stable release with a feed. API, download and
/// validation failures never fall back to an empty feed or overwrite destination.
pub fn fetch(context: &Context<'_>, destination: &Path) -> Result<()> {
    let output = context
        .command("gh")
        .args(["api", "--paginate", "--slurp"])
        .arg(format!("repos/{REPOSITORY}/releases?per_page=100"))
        .output(context.runner)?;
    let pages: Vec<Vec<Release>> =
        serde_json::from_slice(&output.stdout).map_err(|source| Error::Json {
            context: "GitHub releases".into(),
            source,
        })?;
    let mut releases = Vec::new();
    let mut sizes = BTreeMap::new();
    for release in pages.into_iter().flatten() {
        if release.draft || release.prerelease || !stable_tag(&release.tag_name) {
            continue;
        }
        for asset in &release.assets {
            sizes.insert(asset.browser_download_url.clone(), asset.size);
        }
        if release.assets.iter().any(|a| a.name == "appcast.xml") {
            releases.push((stable_version(&release.tag_name[1..])?, release.tag_name));
        }
    }
    let data = if let Some((_, tag)) = releases.into_iter().max() {
        let work = WorkDir::new(context, "voxelite-feed-")?;
        context
            .command("gh")
            .args([
                "release",
                "download",
                &tag,
                "--repo",
                REPOSITORY,
                "--pattern",
                "appcast.xml",
                "--dir",
            ])
            .arg(work.path())
            .run(context.runner)?;
        let source = work.path().join("appcast.xml");
        let items = feed::validate(&source, false, Some(&sizes))?;
        if items[0].version != tag[1..] {
            return Err(invalid(
                "Release appcast does not begin with its own version",
            ));
        }
        read(&source)?
    } else {
        let source = context.path("site/appcast.xml");
        if !feed::validate(&source, true, None)?.is_empty() {
            return Err(invalid(
                "No released appcast exists; refusing to reuse a local nonempty feed",
            ));
        }
        println!("No stable release appcast exists; using the empty feed.");
        read(&source)?
    };
    let destination = context.path(destination);
    if let Some(parent) = destination.parent() {
        mkdir(parent)?;
    }
    write(&destination, data)
}

fn github_output(context: &Context<'_>, generated: bool) -> Result<()> {
    let path = context.environment.get("GITHUB_OUTPUT");
    if path.is_empty() || context.runner.is_dry_run() {
        return Ok(());
    }
    let path = context.path(path);
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&path)
        .map_err(|source| Error::Io {
            path: path.clone(),
            source,
        })?;
    writeln!(file, "generated={generated}").map_err(|source| Error::Io { path, source })
}

/// Escape commit subjects as text, with the same bytes as Python html.escape.
pub fn release_notes_html(notes: &str) -> String {
    let text = notes
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"></head><body>\n<pre style=\"white-space: pre-wrap; font-family: system-ui\">{text}</pre>\n</body></html>\n"
    )
}

/// Generate and verify a signed feed using only stdin for the private key.
/// No network operation runs with the key. Dry runs describe prerequisites but
/// do not require built artifacts, write outputs or execute Sparkle/Apple tools.
pub fn generate(context: &Context<'_>) -> Result<()> {
    github_output(context, false)?;
    let dry = context.runner.is_dry_run();
    let env = context.environment;
    if env.get("SPARKLE_ED_PRIVATE_KEY").is_empty() {
        return Err(invalid(
            "SPARKLE_ED_PRIVATE_KEY is required for a release appcast.",
        ));
    }
    if !dry && !context.path("dist/.notarized").is_file() {
        return Err(invalid(
            "A notarized DMG is required for a release appcast.",
        ));
    }
    if !dry
        && !context
            .path("dist/Voxelite.app/Contents/Frameworks/Sparkle.framework")
            .is_dir()
    {
        return Err(invalid("The release must embed Sparkle."));
    }
    let tag = env.get("TAG");
    if tag.is_empty() {
        return Err(invalid("TAG: Set TAG to the GitHub release tag"));
    }
    if env.or("GITHUB_REPOSITORY", REPOSITORY) != REPOSITORY {
        return Err(invalid("Invalid GitHub repository"));
    }
    if !stable_tag(tag) {
        return Err(invalid("Invalid release tag"));
    }
    let prefix = format!("{DOWNLOAD}{tag}/");
    let work = WorkDir::new(context, "voxelite-appcast.")?;
    context.plan("require dist/.notarized, embedded Sparkle, matching bundle version, and a valid older dist/previous-appcast.xml");
    if !dry {
        let info = files::read_plist(&context.path("dist/Voxelite.app/Contents/Info.plist"))?;
        let version = files::plist_string(&info, "CFBundleVersion")?;
        if tag != format!("v{version}") {
            return Err(invalid(
                "Release tag does not match the packaged bundle version",
            ));
        }
        let previous = feed::validate(&context.path("dist/previous-appcast.xml"), true, None)?;
        if let Some(first) = previous.first()
            && stable_version(version)? <= stable_version(&first.version)?
        {
            return Err(invalid(
                "Release version must be newer than the published appcast",
            ));
        }
    }
    context.plan("verify dist/Voxelite.dmg against dist/Voxelite.dmg.sha256");
    if !dry {
        let checksum = String::from_utf8(read(&context.path("dist/Voxelite.dmg.sha256"))?)
            .map_err(|_| invalid("Invalid DMG checksum file"))?;
        let expected = format!(
            "{}  Voxelite.dmg\n",
            files::sha256(&context.path("dist/Voxelite.dmg"))?
        );
        if checksum != expected {
            return Err(invalid("Voxelite.dmg: SHA-256 checksum did not match"));
        }
    }
    context
        .command("xcrun")
        .args(["stapler", "validate", "dist/Voxelite.dmg"])
        .run(context.runner)?;
    let archives = work.path().join("archives");
    let output = work.path().join("appcast.xml");
    let notes = archives.join("Voxelite.html");
    context.plan("copy final DMG and previous feed into private workspace; HTML-escape dist/RELEASE_NOTES.md to archives/Voxelite.html");
    if !dry {
        mkdir(&archives)?;
        copy(
            &context.path("dist/Voxelite.dmg"),
            &archives.join("Voxelite.dmg"),
        )?;
        copy(&context.path("dist/previous-appcast.xml"), &output)?;
        let markdown = String::from_utf8(read(&context.path("dist/RELEASE_NOTES.md"))?)
            .map_err(|_| invalid("Release notes are not UTF-8"))?;
        write(&notes, release_notes_html(&markdown))?;
    }
    let input = format!("{}\n", env.get("SPARKLE_ED_PRIVATE_KEY"));
    context
        .command("dist/sparkle/bin/generate_appcast")
        .args([
            "--ed-key-file",
            "-",
            "--maximum-deltas",
            "0",
            "--maximum-versions",
            "0",
            "--download-url-prefix",
            &prefix,
            "--release-notes-url-prefix",
            &prefix,
            "--link",
            "https://voxelite.worldwidewebb.co",
            "-o",
        ])
        .arg(&output)
        .arg(&archives)
        .secret_stdin(input.as_bytes())
        .output(context.runner)
        .map_err(|_| invalid("Appcast generation failed; raw key diagnostics suppressed."))?;
    context
        .command("dist/sparkle/bin/sign_update")
        .args(["--ed-key-file", "-", "--verify"])
        .arg(&output)
        .secret_stdin(input.as_bytes())
        .output(context.runner)
        .map_err(|_| {
            invalid("Appcast signature verification failed; raw key diagnostics suppressed.")
        })?;
    context.plan("copy signed Voxelite.html to dist; validate every feed item, local asset length, bundle metadata and preserved history");
    if !dry {
        copy(&notes, &context.path("dist/Voxelite.html"))?;
        super::check_appcast(
            &output,
            false,
            Some(&context.path("dist")),
            Some(&context.path("dist/previous-appcast.xml")),
        )?;
    }
    context.plan("copy signed XML bytes unchanged to site/appcast.xml; chmod XML/HTML 0644; append generated=true to GITHUB_OUTPUT when set");
    if !dry {
        mkdir(&context.path("site"))?;
        copy(&output, &context.path("site/appcast.xml"))?;
        for path in ["site/appcast.xml", "dist/Voxelite.html"] {
            files::public_permissions(&context.path(path))?;
        }
    }
    github_output(context, true)?;
    context.plan(format_args!(
        "remove temporary directory {}",
        work.path().display()
    ));
    println!(
        "{} site/appcast.xml for upload as a release asset. The site deploy must copy this file unchanged to /appcast.xml.",
        if dry { "Would generate" } else { "Generated" }
    );
    Ok(())
}
