//! Release workflow regression tests, executing the actual run blocks with Rust fixtures.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
    sync::OnceLock,
};
use xtask::{
    cmd::{Runner, SystemRunner},
    macos,
};

const WORKFLOW: &str = include_str!("../../.github/workflows/release-macos.yml");
fn workflow_step(name: &str) -> String {
    let lines: Vec<_> = WORKFLOW.lines().collect();
    let start = lines
        .iter()
        .position(|line| *line == format!("      - name: {name}"))
        .unwrap();
    let start = lines[start..]
        .iter()
        .position(|line| *line == "        run: |")
        .unwrap()
        + start
        + 1;
    lines[start..]
        .iter()
        .take_while(|line| line.is_empty() || line.starts_with("          "))
        .map(|line| line.get(10..).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}
fn tool() -> &'static Path {
    static TOOL: OnceLock<tempfile::TempDir> = OnceLock::new();
    TOOL.get_or_init(|| {
        let temp = tempfile::tempdir().unwrap();
        let output = Command::new("rustc")
            .args(["--edition=2024", "--crate-name=workflow_tool"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/workflow_tool.rs"))
            .arg("-o")
            .arg(temp.path().join("tool"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        temp
    })
    .path()
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("tools")).unwrap();
    for name in ["gh", "cargo", "secret-echo"] {
        fs::copy(tool().join("tool"), temp.path().join("tools").join(name)).unwrap();
    }
    temp
}
fn shell(root: &Path, step: &str) -> Command {
    let mut command = Command::new("bash");
    command
        .args(["-euo", "pipefail", "-c", &workflow_step(step)])
        .current_dir(root)
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("tools").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("CARGO", root.join("tools/cargo"))
        .env("REAL_XTASK", env!("CARGO_BIN_EXE_xtask"));
    for name in macos::SECRETS {
        command.env_remove(name);
    }
    command
}
fn success(output: &Output, expected: bool) {
    assert_eq!(
        output.status.success(),
        expected,
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn release_tags() {
    for tag in [
        "v0.1.0",
        "v12.34.56",
        "main",
        "v1",
        "v01.2.3",
        "v1.2.3-rc.1",
        "v1.2.3\nmain",
        "--help",
        "$(false)",
    ] {
        let root = fixture();
        let output = shell(root.path(), "Validate release tag")
            .env("TAG", tag)
            .output()
            .unwrap();
        success(&output, matches!(tag, "v0.1.0" | "v12.34.56"));
        assert_eq!(output.status.success(), macos::stable_tag(tag));
    }
    assert!(
        WORKFLOW.contains("ref: refs/tags/${{ env.TAG }}")
            && WORKFLOW.contains("          fetch-depth: 0")
    );
}
#[test]
fn self_hosted_guards_and_pins() {
    let guard = "    if: github.actor == '0x63616c' && (github.event_name != 'pull_request' || github.event.pull_request.head.repo.full_name == github.repository)";
    for path in
        fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows")).unwrap()
    {
        let path = path.unwrap().path();
        if path.extension().is_none_or(|e| e != "yml") {
            continue;
        }
        let text = fs::read_to_string(path).unwrap();
        assert!(!text.contains("pull_request_target:"));
        if text.contains("self-hosted") {
            assert!(text.lines().any(|line| line == guard));
        }
        for line in text.lines().filter(|line| line.contains("uses: actions/")) {
            let pin = line
                .split_once('@')
                .unwrap()
                .1
                .split_whitespace()
                .next()
                .unwrap();
            assert!(
                pin.len() == 40
                    && pin
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            );
        }
    }
}
#[test]
fn release_build_precedes_credentials() {
    let position = |name: &str| WORKFLOW.find(&format!("      - name: {name}")).unwrap();
    let package = position("Sign and package");
    assert!(position("Build app without signing credentials") < package);
    assert!(!WORKFLOW[..package].contains("secrets."));
    let build = workflow_step("Build app without signing credentials");
    for command in [
        "cargo build --release --locked -p xtask",
        "cargo build --release --locked -p voxelite-mcp",
        "target/release/xtask version --check-tag",
    ] {
        assert!(build.contains(command));
    }
    assert!(build.find("version --check-tag").unwrap() < build.find("-p voxelite-app").unwrap());
    let sign = workflow_step("Sign and package");
    assert!(
        sign.contains("target/release/xtask macos package --skip-build")
            && !sign.contains("cargo ")
    );
    assert!(
        package < position("Generate release notes")
            && position("Generate release notes") < position("Generate signed update feed")
    );
    let appcast = &WORKFLOW[position("Generate signed update feed")..position("Publish release")];
    assert!(
        appcast.contains("run: target/release/xtask macos appcast") && !appcast.contains("cargo ")
    );
}
#[test]
fn tag_must_match_cargo_before_building() {
    for (tag, expected) in [("v1.2.4", false), ("v1.2.3", true)] {
        let root = fixture();
        let built = root.path().join("built");
        success(
            &shell(root.path(), "Build app without signing credentials")
                .env("TAG", tag)
                .env("BUILT", &built)
                .output()
                .unwrap(),
            expected,
        );
        assert_eq!(built.exists(), expected);
    }
}
#[test]
fn site_release_trigger_and_shared_concurrency() {
    let workflow = include_str!("../../.github/workflows/deploy-site.yml");
    for text in [
        "workflows: [\"Release macOS\"]",
        "github.event.workflow_run.conclusion == 'success'",
        "github.event.workflow_run.actor.login == '0x63616c'",
        "github.event.workflow_run.head_repository.full_name == github.repository",
        "group: deploy-site-${{ github.ref }}",
        "cargo xtask macos fetch-appcast site/appcast.xml",
    ] {
        assert!(workflow.contains(text));
    }
}
#[test]
fn release_requires_all_apple_credentials_before_packaging() {
    let names = [
        "MACOS_CERTIFICATE_P12_BASE64",
        "MACOS_CERTIFICATE_PASSWORD",
        "APPLE_API_KEY_P8_BASE64",
        "APPLE_API_KEY_ID",
        "APPLE_API_ISSUER_ID",
    ];
    for missing in names.into_iter().map(Some).chain([None]) {
        let root = fixture();
        fs::create_dir_all(root.path().join("target/release")).unwrap();
        fs::copy(
            tool().join("tool"),
            root.path().join("target/release/xtask"),
        )
        .unwrap();
        let mut cmd = shell(root.path(), "Sign and package");
        cmd.env("TAG", "v1.2.3");
        for name in names {
            cmd.env(
                name,
                if Some(name) == missing {
                    ""
                } else {
                    "fixture value"
                },
            );
        }
        let output = cmd.output().unwrap();
        success(&output, missing.is_none());
        assert_eq!(root.path().join("packaged").exists(), missing.is_none());
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains("fixture value")
                && !String::from_utf8_lossy(&output.stderr).contains("fixture value")
        );
    }
}
#[test]
fn publication_is_staged_and_immutable() {
    for (generated, failure) in [(true, ""), (false, ""), (true, "create"), (true, "upload")] {
        let root = fixture();
        let log = root.path().join("calls");
        let output = shell(root.path(), "Publish release")
            .env("TAG", "v1.2.3")
            .env("APPCAST_GENERATED", generated.to_string())
            .env("CALLS", &log)
            .env("FAIL_AT", failure)
            .output()
            .unwrap();
        success(&output, generated && failure.is_empty());
        let calls: Vec<Vec<String>> = fs::read_to_string(log)
            .unwrap_or_default()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        if !generated {
            assert!(calls.is_empty());
            continue;
        }
        assert!(calls[0].contains(&"--draft".into()) && calls[0].contains(&"--verify-tag".into()));
        assert_eq!(
            calls[0][calls[0].iter().position(|a| a == "--notes-file").unwrap() + 1],
            "dist/RELEASE_NOTES.md"
        );
        if failure == "create" {
            assert_eq!(calls.len(), 1);
            continue;
        }
        assert_eq!(calls[1][..3], ["release", "upload", "v1.2.3"]);
        assert!(!calls[1].contains(&"--clobber".into()));
        for asset in [
            "dist/Voxelite.dmg",
            "dist/Voxelite.dmg.sha256",
            "dist/Voxelite.html",
            "site/appcast.xml",
        ] {
            assert!(calls[1].contains(&asset.into()));
        }
        if failure == "upload" {
            assert_eq!(calls.len(), 2);
        } else {
            assert_eq!(calls[2], ["release", "edit", "v1.2.3", "--draft=false"]);
        }
    }
}
#[test]
fn cli_rejects_wrong_local_dmg_length() {
    let root = fixture();
    let dist = root.path().join("dist");
    let bundle = dist.join("Voxelite.app/Contents");
    fs::create_dir_all(&bundle).unwrap();
    let mut info = plist::Dictionary::new();
    for (key, value) in [
        ("CFBundleVersion", "1.2.3"),
        ("CFBundleShortVersionString", "1.2.3"),
        ("LSMinimumSystemVersion", "13.0"),
    ] {
        info.insert(key.into(), value.into());
    }
    plist::Value::Dictionary(info)
        .to_file_xml(bundle.join("Info.plist"))
        .unwrap();
    fs::write(dist.join("Voxelite.html"), "fixture notes").unwrap();
    let signature =
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
    let feed = root.path().join("feed.xml");
    fs::write(&feed, format!("<rss version=\"2.0\" xmlns:s=\"http://www.andymatuschak.org/xml-namespaces/sparkle\"><channel><item><s:version>1.2.3</s:version><s:shortVersionString>1.2.3</s:shortVersionString><s:minimumSystemVersion>13.0</s:minimumSystemVersion><enclosure url=\"{}v1.2.3/Voxelite.dmg\" length=\"11\" s:edSignature=\"{signature}\"/><s:releaseNotesLink s:length=\"13\" s:edSignature=\"{signature}\">{}v1.2.3/Voxelite.html</s:releaseNotesLink></item></channel></rss>", macos::DOWNLOAD, macos::DOWNLOAD)).unwrap();
    for (data, valid) in [("fixture dmg", true), ("altered", false)] {
        fs::write(dist.join("Voxelite.dmg"), data).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["macos", "check-appcast"])
            .arg(&feed)
            .arg("--release-dir")
            .arg(&dist)
            .arg("--previous")
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../site/appcast.xml"))
            .output()
            .unwrap();
        success(&output, valid);
        if !valid {
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("Invalid appcast: Appcast DMG length does not match")
            );
        }
    }
}
#[test]
fn secret_stdin_uses_a_pipe_and_diagnostics_are_redacted() {
    let root = fixture();
    let command = macos::clean(xtask::cmd::Command::new(
        root.path().join("tools/secret-echo"),
    ))
    .secret_stdin(b"fixture secret stdin\n".to_vec());
    let error = command.output(&SystemRunner::default()).unwrap_err();
    assert!(error.to_string().contains("17") && error.to_string().contains("[REDACTED]"));
    assert!(!format!("{command:?} {error:?}").contains("fixture secret stdin"));
}
#[test]
fn interrupted_runner_kills_and_reaps_child_but_allows_cleanup() {
    let root = fixture();
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // Isolated signal registration isn't required here: use a thread to request
    // cancellation, avoiding any signal handler changes in this parallel test host.
    let runner = SystemRunner::default().with_interruption(flag.clone());
    let start = std::time::Instant::now();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        flag.store(143, std::sync::atomic::Ordering::SeqCst);
    });
    let output = xtask::cmd::Command::new("sleep")
        .arg("30")
        .output(&runner)
        .unwrap_err();
    thread.join().unwrap();
    assert!(output.to_string().contains("143"));
    assert!(start.elapsed().as_secs() < 5);
    assert_eq!(runner.interruption_code(), 143);
    let cleanup = xtask::cmd::Command::new(root.path().join("tools/gh"))
        .args(["release", "edit"])
        .during_cleanup();
    cleanup.run(&runner).unwrap();
}

#[test]
fn inherited_credentials_helper() {
    let Some(tool) = std::env::var_os("XTASK_TEST_SECRET_TOOL") else {
        return;
    };
    for name in macos::SECRETS {
        assert_eq!(std::env::var(name).unwrap(), "fixture inherited secret");
    }
    let command = macos::clean(xtask::cmd::Command::new(tool))
        .secret_stdin(b"fixture secret stdin\n".to_vec());
    let error = command
        .output(&SystemRunner::default())
        .unwrap_err()
        .to_string();
    // The fixture checks the real process environment before intentionally exiting 17.
    assert!(error.contains("17") && !error.contains("fixture secret stdin"));
}

#[test]
fn real_child_removes_all_inherited_credentials() {
    let root = fixture();
    let mut child = Command::new(std::env::current_exe().unwrap());
    child
        .args(["--exact", "inherited_credentials_helper", "--nocapture"])
        .env(
            "XTASK_TEST_SECRET_TOOL",
            root.path().join("tools/secret-echo"),
        );
    for name in macos::SECRETS {
        child.env(name, "fixture inherited secret");
    }
    success(&child.output().unwrap(), true);
}
