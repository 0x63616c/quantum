use super::*;
use crate::{
    cmd::{Output, Status},
    testing::RecordingRunner,
};

fn fixture() -> (tempfile::TempDir, Workspace) {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "Cargo.toml", "[workspace]\n");
    write(temp.path(), "xtask/Cargo.toml", "");
    let workspace = Workspace::at(temp.path()).unwrap();
    (temp, workspace)
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

#[test]
fn local_urls_preserve_paths_and_decode_without_form_encoding() {
    for (target, expected) in [
        ("../a%20b.md#heading", "../a b.md"),
        ("a+b.png?raw=1", "a+b.png"),
        ("#heading", ""),
        ("?query", ""),
        ("/absolute/file", "/absolute/file"),
        ("a%23b%3Fc.svg", "a#b?c.svg"),
        ("a%ZZ.png", "a%ZZ.png"),
        ("caf%C3%A9.md", "café.md"),
        ("invalid%FF.md", "invalid�.md"),
        ("dir/a:b.md", "dir/a:b.md"),
        ("///absolute/file", "/absolute/file"),
        ("//?query", ""),
    ] {
        assert_eq!(local_path(target).as_deref(), Some(expected), "{target}");
    }
    for target in [
        "https://example.com/a",
        "//example.com/a",
        "mailto:a@example.com",
        "git+ssh:foo",
        "HTTP:foo",
    ] {
        assert_eq!(local_path(target), None, "{target}");
    }
}

#[test]
fn links_resolve_from_each_document_and_require_links() {
    let (_temp, workspace) = fixture();
    write(workspace.root(), "docs/a b.md", "");
    write(
        workspace.root(),
        "docs/glossary.md",
        "[file](a%20b.md#part) [anchor](#part) [web](https://example.com) [query](?q)",
    );
    write(
        workspace.root(),
        "docs/adr/0003-product-direction.md",
        "[file](../a%20b.md)",
    );
    assert!(check_links(&workspace).unwrap().is_empty());
    write(
        workspace.root(),
        "docs/adr/0003-product-direction.md",
        "[missing](a%20b.md)",
    );
    assert_eq!(
        check_links(&workspace).unwrap(),
        ["docs/adr/0003-product-direction.md: missing local link target: a%20b.md"]
    );
    write(workspace.root(), "docs/glossary.md", "No links.");
    assert_eq!(check_links(&workspace).unwrap().len(), 2);
}

#[test]
fn image_targets_include_plain_links_all_extensions_and_encoded_paths() {
    let (_temp, workspace) = fixture();
    for extension in ["png", "JPG", "jpeg", "gif", "svg", "webp"] {
        let path = format!("docs/a b.{extension}");
        write(workspace.root(), &path, "image");
        let source = format!("![](a%20b.{extension}?raw#fragment) [download](a%20b.{extension})");
        assert_eq!(
            image_errors(&workspace, &workspace.path("docs/file.md"), &source).unwrap(),
            (2, vec![])
        );
    }
    let (count, errors) = image_errors(
        &workspace,
        &workspace.path("README.md"),
        "![x](missing.png) [other](missing.txt) ![](https://host/a.png) ![](//host/b.png)",
    )
    .unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        errors,
        [format!(
            "Missing documentation image: {}",
            workspace.path("missing.png").display()
        )]
    );
    write(workspace.root(), "screenshots/a.png", "image");
    assert_eq!(
        image_errors(
            &workspace,
            &workspace.path("docs/file.md"),
            "![](../screenshots/a.png)"
        )
        .unwrap()
        .1
        .len(),
        1
    );
    std::fs::create_dir_all(workspace.path("directory.png")).unwrap();
    assert_eq!(
        image_errors(
            &workspace,
            &workspace.path("README.md"),
            "![](directory.png)"
        )
        .unwrap()
        .1
        .len(),
        1
    );
}

#[cfg(unix)]
#[test]
fn symlinks_into_capture_output_are_rejected() {
    let (_temp, workspace) = fixture();
    write(workspace.root(), "screenshots/a.png", "image");
    std::os::unix::fs::symlink(
        workspace.path("screenshots/a.png"),
        workspace.path("alias.png"),
    )
    .unwrap();
    assert_eq!(
        image_errors(&workspace, &workspace.path("README.md"), "![](alias.png)")
            .unwrap()
            .1
            .len(),
        1
    );
}

#[cfg(unix)]
#[test]
fn document_discovery_does_not_follow_directory_symlinks() {
    let (_temp, workspace) = fixture();
    write(workspace.root(), "docs/real/file.md", "");
    write(workspace.root(), "outside/skipped.md", "");
    std::os::unix::fs::symlink(workspace.path("docs"), workspace.path("docs/real/loop")).unwrap();
    std::os::unix::fs::symlink(workspace.path("outside"), workspace.path("docs/linked")).unwrap();
    assert_eq!(
        super::super::descendants(&workspace, "docs", "*.md").unwrap(),
        [workspace.path("docs/real/file.md")]
    );
}

#[test]
fn capture_ignore_policy_checks_all_samples_and_durable_images() {
    let (_temp, workspace) = fixture();
    let runner = RecordingRunner::default();
    assert_eq!(
        check_ignored_assets(&workspace, &runner).unwrap(),
        ["expected at least one PNG in docs/images"]
    );
    assert!(runner.commands.borrow().is_empty());
    write(workspace.root(), "docs/images/example.png", "image");
    let expected = "screenshots/app-dark.png\nscreenshots/site/final/verification.json\nscreenshots/review-1/tools.png\n";
    let runner = RecordingRunner::stdout(expected.as_bytes());
    assert!(
        check_ignored_assets(&workspace, &runner)
            .unwrap()
            .is_empty()
    );
    runner.assert_args(
        0,
        &[
            "check-ignore",
            "--no-index",
            "--",
            "screenshots/app-dark.png",
            "screenshots/site/final/verification.json",
            "screenshots/review-1/tools.png",
            "docs/images/example.png",
        ],
    );
    assert_eq!(
        runner.commands.borrow()[0].0.directory(),
        Some(workspace.root())
    );
    for incorrect in [
        "screenshots/app-dark.png\n".to_owned(),
        format!("{expected}docs/images/example.png\n"),
    ] {
        assert_eq!(
            check_ignored_assets(&workspace, &RecordingRunner::stdout(incorrect.as_bytes()))
                .unwrap()
                .len(),
            1
        );
    }
    let runner = RecordingRunner::default();
    runner.results.borrow_mut().push_back(Ok(Output {
        status: Status::from_code(128),
        stderr: b"not a repository".to_vec(),
        ..Output::default()
    }));
    assert!(
        check_ignored_assets(&workspace, &runner)
            .unwrap_err()
            .to_string()
            .contains("not a repository")
    );
}

#[test]
fn asset_scan_covers_original_document_set_and_requires_an_image() {
    let (_temp, workspace) = fixture();
    for document in [
        "README.md",
        "AGENTS.md",
        "examples/README.md",
        "site/README.md",
        "design/ART-CHECKLIST.md",
        "docs/nested/check.md",
        "crates/a/nested/README.md",
    ] {
        write(workspace.root(), document, "![](missing.png)");
    }
    // Other Markdown files in crates and other root documents were never scanned.
    write(workspace.root(), "crates/a/other.md", "![](missing.png)");
    write(workspace.root(), "OTHER.md", "![](missing.png)");
    let errors = check_assets(&workspace, &RecordingRunner::default()).unwrap();
    assert_eq!(
        errors
            .iter()
            .filter(|e| e.starts_with("Missing documentation image:"))
            .count(),
        7
    );
    for document in super::super::paths(&workspace, "**/*.md").unwrap() {
        std::fs::write(document, "").unwrap();
    }
    assert!(
        check_assets(&workspace, &RecordingRunner::default())
            .unwrap()
            .contains(&"expected at least one local documentation image".into())
    );
}
