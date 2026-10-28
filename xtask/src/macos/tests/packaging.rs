use super::support::*;
use crate::{
    cmd::{Runner, SystemRunner},
    macos::{self, files, package, sign},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[test]
fn no_secrets_reach_build_or_tools() {
    let mut f = Fixture::new();
    f.cert();
    f.notary();
    f.env.set("SPARKLE_ED_PRIVATE_KEY", SECRET_KEY);
    f.package(false).unwrap();
    assert!(f.path("dist/.notarized").exists());
    let expected = format!(
        "{}  Voxelite.dmg\n",
        files::sha256(&f.path("dist/Voxelite.dmg")).unwrap()
    );
    assert_eq!(
        fs::read_to_string(f.path("dist/Voxelite.dmg.sha256")).unwrap(),
        expected
    );
    for (command, _) in f.runner.events.borrow().iter() {
        for value in [
            "fixture password",
            "fixture random password",
            "fixture ID",
            "fixture issuer",
            SECRET_KEY,
            &"A".repeat(40),
        ] {
            assert!(!format!("{command:?}").contains(value));
        }
    }
    // Independently verify the release checksum from a download directory.
    let download = tempfile::tempdir().unwrap();
    for name in ["Voxelite.dmg", "Voxelite.dmg.sha256"] {
        fs::copy(f.path(&format!("dist/{name}")), download.path().join(name)).unwrap();
    }
    assert!(
        std::process::Command::new("shasum")
            .args(["-a", "256", "--check", "Voxelite.dmg.sha256"])
            .current_dir(download.path())
            .output()
            .unwrap()
            .status
            .success()
    );
}
#[test]
fn default_public_key_and_matching_override() {
    for override_key in [false, true] {
        let mut f = Fixture::new();
        let raw = fs::read_to_string(f.path("packaging/macos/sparkle_public_ed_key.txt")).unwrap();
        assert_eq!(raw, format!("{}\n", raw.trim()));
        assert_eq!(STANDARD.decode(raw.trim()).unwrap().len(), 32);
        if override_key {
            f.env.set("SPARKLE_PUBLIC_ED_KEY", raw.trim());
        }
        f.package(false).unwrap();
        let info = files::read_plist(&f.path("dist/Voxelite.app/Contents/Info.plist")).unwrap();
        assert_eq!(
            files::plist_string(&info, "SUPublicEDKey").unwrap(),
            raw.trim()
        );
        let feed = files::plist_string(&info, "SUFeedURL").unwrap();
        assert_eq!(feed, "https://voxelite.worldwidewebb.co/appcast.xml");
        let brand = fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../brand.toml"),
        )
        .unwrap();
        assert!(
            brand
                .lines()
                .any(|l| l.starts_with("sparkle_appcast_url ") && l.contains(feed))
        );
        for (key, expected) in [
            ("SUEnableAutomaticChecks", true),
            ("SUAutomaticallyUpdate", false),
            ("SURequireSignedFeed", true),
            ("SUVerifyUpdateBeforeExtraction", true),
        ] {
            assert_eq!(info[key].as_boolean(), Some(expected));
        }
        let signatures = f.runner.calls("codesign");
        assert!(!signatures.is_empty());
        for args in signatures
            .iter()
            .filter(|a| !a.contains(&"--verify".into()))
        {
            assert!(!args.contains(&"--options".into()) && !args.contains(&"--deep".into()));
        }
    }
}
#[test]
fn mismatched_public_override_and_missing_default_fail() {
    let mut f = Fixture::new();
    f.env
        .set("SPARKLE_PUBLIC_ED_KEY", STANDARD.encode([0u8; 32]));
    assert!(
        f.package(false)
            .unwrap_err()
            .to_string()
            .starts_with("SPARKLE_PUBLIC_ED_KEY differs")
    );
    let valid = fs::read_to_string(f.path("packaging/macos/sparkle_public_ed_key.txt")).unwrap();
    fs::remove_file(f.path("packaging/macos/sparkle_public_ed_key.txt")).unwrap();
    f.env.set("SPARKLE_PUBLIC_ED_KEY", valid);
    assert!(f.package(false).is_err());
}
#[test]
fn skip_build_and_missing_mcp_executable() {
    for mcp in [true, false] {
        let mut f = Fixture::new();
        f.cert();
        write(f.path("target/release/voxelite"), "fixture app");
        if mcp {
            write(f.path("target/release/voxelite-mcp"), "fixture server");
        }
        assert_eq!(f.package(true).is_ok(), mcp);
        assert!(f.runner.calls("cargo").is_empty());
        if !mcp {
            assert!(f.runner.calls("codesign").is_empty());
        }
    }
}
#[test]
fn bundled_executables_and_sparkle_signed_inside_out() {
    let mut f = Fixture::new();
    f.cert();
    f.notary();
    f.package(false).unwrap();
    assert!(
        f.runner.calls("cargo").contains(
            &["build", "--release", "--locked", "-p", "voxelite-mcp"]
                .map(str::to_owned)
                .to_vec()
        )
    );
    let signatures: Vec<_> = f
        .runner
        .calls("codesign")
        .into_iter()
        .filter(|a| !a.contains(&"--verify".into()))
        .collect();
    let names: Vec<_> = signatures
        .iter()
        .map(|a| {
            std::path::Path::new(a.last().unwrap())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        })
        .collect();
    assert_eq!(
        names,
        [
            "Installer.xpc",
            "Downloader.xpc",
            "Autoupdate",
            "Updater.app",
            "Sparkle.framework",
            "voxelite-mcp",
            "Voxelite",
            "Voxelite.app",
            "Voxelite.dmg"
        ]
    );
    for args in &signatures[..8] {
        assert_eq!(
            args[args.iter().position(|a| a == "--sign").unwrap() + 1],
            "A".repeat(40)
        );
        assert_eq!(
            args[args.iter().position(|a| a == "--options").unwrap() + 1],
            "runtime"
        );
        assert!(args.contains(&"--timestamp".into()) && !args.contains(&"--deep".into()));
    }
    for (source, target) in [("voxelite", "Voxelite"), ("voxelite-mcp", "voxelite-mcp")] {
        assert_eq!(
            fs::read(f.path(&format!("target/release/{source}"))).unwrap(),
            fs::read(f.path(&format!("dist/Voxelite.app/Contents/MacOS/{target}"))).unwrap()
        );
    }
    let events = f.runner.events.borrow();
    let positions = |program: &str, word: &str| {
        events
            .iter()
            .enumerate()
            .filter(|(_, (c, _))| c.program() == program && c.arguments().any(|a| a == word))
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    };
    let stapled = positions("xcrun", "staple");
    let dmg = positions("hdiutil", "create")[0];
    assert!(stapled[0] < dmg && dmg < stapled[1]);
}
#[test]
fn adhoc_and_partial_credentials() {
    for mode in [0, 1, 2] {
        let mut f = Fixture::new();
        if mode == 1 {
            f.env.set(
                "MACOS_CERTIFICATE_P12_BASE64",
                STANDARD.encode(b"fixture certificate"),
            );
        }
        if mode == 2 {
            f.cert();
            f.notary();
            f.env.set("MACOS_SIGNING_MODE", "adhoc");
        }
        f.package(false).unwrap();
        assert!(!f.path("dist/.notarized").exists());
        assert!(f.runner.calls("security").is_empty());
    }
}
#[test]
fn empty_keychain_list_and_lowercase_fingerprint() {
    let mut f = Fixture::new();
    f.cert();
    f.env.set("MACOS_SIGNING_IDENTITY", "a".repeat(40));
    f.runner.set("keychains", "");
    f.package(false).unwrap();
    assert_eq!(
        f.runner.calls("security").last().unwrap(),
        &["list-keychains", "-d", "user", "-s"]
    );
}
#[test]
fn malformed_keychain_list_stops_before_mutation() {
    let mut f = Fixture::new();
    f.cert();
    f.runner.set("keychains", "\"unterminated");
    assert!(f.package(false).is_err());
    assert!(
        !f.runner
            .calls("security")
            .iter()
            .any(|a| a[0] == "create-keychain")
    );
}
#[test]
fn private_failures_clean_keychain_and_hide_diagnostics() {
    for operation in [
        "security:create-keychain",
        "security:set-keychain-settings",
        "security:unlock-keychain",
        "security:import",
        "security:set-key-partition-list",
        "security:find-identity",
        "codesign",
        "xcrun:notarytool",
        "spctl",
    ] {
        let mut f = Fixture::new();
        f.cert();
        f.notary();
        f.runner.set("fail", operation);
        let error = f.package(false).unwrap_err().to_string();
        assert!(
            !error.contains(DIAGNOSTIC) && !error.contains(SECRET_KEY),
            "{error}"
        );
        let events = f.runner.calls("security");
        assert_eq!(events[events.len() - 2][0], "delete-keychain");
        assert_eq!(
            events.last().unwrap()[..4],
            ["list-keychains", "-d", "user", "-s"]
        );
        assert!(!f.path("dist/.notarized").exists());
    }
}
#[test]
fn cleanup_failures_and_panics_invalidate_marker() {
    for failure in ["security:delete-keychain", "restore"] {
        let mut f = Fixture::new();
        f.cert();
        f.notary();
        if failure == "restore" {
            f.runner.set("fail_restore", "1");
        } else {
            f.runner.set("fail", failure);
        }
        assert!(matches!(f.package(false), Err(crate::Error::Cleanup(_))));
        assert!(!f.path("dist/.notarized").exists());
        if failure.ends_with("delete-keychain") {
            assert_eq!(
                f.runner.calls("security").last().unwrap()[0],
                "list-keychains"
            );
        }
    }
    let mut f = Fixture::new();
    f.cert();
    f.runner.set("panic", "security:import");
    assert!(catch_unwind(AssertUnwindSafe(|| f.package(false))).is_err());
    assert!(
        f.runner
            .calls("security")
            .iter()
            .any(|a| a[0] == "delete-keychain")
    );
    assert_eq!(fs::read_dir(f.path("tmp")).unwrap().count(), 0);
    assert!(!f.path("dist/.notarized").exists());
}
#[test]
fn failure_cannot_reuse_notarized_marker() {
    let mut f = Fixture::new();
    f.cert();
    f.notary();
    f.package(false).unwrap();
    f.runner.set("fail", "cargo");
    assert!(f.package(false).is_err());
    assert!(!f.path("dist/.notarized").exists());
    write(f.path("dist/.notarized"), "");
    f.env.set("VERSION", "main");
    assert!(f.package(false).is_err());
    assert!(!f.path("dist/.notarized").exists());
}
#[test]
fn invalid_inputs_and_notarization_rejection() {
    for (name, value) in [
        ("VERSION", "main"),
        ("MACOS_SIGNING_MODE", "wrong"),
        ("SPARKLE_PUBLIC_ED_KEY", "invalid"),
        ("MACOS_CERTIFICATE_P12_BASE64", "invalid"),
        ("notary_status", "Invalid"),
        ("corrupt_archive", "1"),
    ] {
        let mut f = Fixture::new();
        f.cert();
        f.notary();
        if name.chars().next().unwrap().is_uppercase() {
            f.env.set(name, value);
        } else {
            f.runner.set(name, value);
        }
        assert!(f.package(false).is_err(), "{name}");
        assert!(!f.path("dist/.notarized").exists());
        if name == "corrupt_archive" {
            assert!(f.runner.calls("tar").is_empty());
        }
    }
    let f = Fixture::new();
    assert_eq!(
        package::package(
            &f.context(),
            &package::Options {
                is_macos: false,
                ..Default::default()
            }
        )
        .unwrap_err()
        .to_string(),
        "macOS packaging requires macOS and Xcode."
    );
}
#[test]
fn identity_selection_and_keychain_parser_contracts() {
    let digest = "A".repeat(40);
    let one = format!("1) {digest} \"Developer ID Application: Calum\"");
    for requested in [
        "",
        "Developer ID Application: Calum",
        &digest,
        &digest.to_lowercase(),
    ] {
        assert_eq!(sign::select_identity(&one, requested).unwrap(), digest);
    }
    for output in [
        "",
        "invalid",
        &format!("{one}\n{one}"),
        &one.replace("Developer ID Application", "Apple Development"),
    ] {
        assert!(sign::select_identity(output, "").is_err());
    }
    assert!(sign::select_identity(&one, "wrong").is_err());
    assert_eq!(
        sign::keychain_paths(" \"/Users/a b/login.keychain-db\"\n '/tmp/a#b' /tmp/c\\ d").unwrap(),
        ["/Users/a b/login.keychain-db", "/tmp/a#b", "/tmp/c d"]
    );
    assert!(sign::keychain_paths("trailing\\").is_err());
}
#[test]
fn dry_runs_do_not_modify_artifacts_or_require_macos() {
    for mode in ["adhoc", "auto"] {
        let mut f = Fixture::new();
        f.cert();
        f.notary();
        f.env.set("MACOS_SIGNING_MODE", mode);
        write(f.path("dist/sentinel"), "keep");
        write(f.path("dist/.notarized"), "keep");
        let context = macos::Context {
            root: f.root(),
            runner: &SystemRunner::dry_run(),
            environment: &f.env,
        };
        package::package(
            &context,
            &package::Options {
                is_macos: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(fs::read_to_string(f.path("dist/sentinel")).unwrap(), "keep");
        assert_eq!(
            fs::read_to_string(f.path("dist/.notarized")).unwrap(),
            "keep"
        );
        assert_eq!(fs::read_dir(f.path("tmp")).unwrap().count(), 0);
    }
}

// Isolated subprocess: no process-global signal or environment changes in the test host.
#[test]
fn signal_helper() {
    let Ok(signal) = std::env::var("XTASK_TEST_SIGNAL") else {
        return;
    };
    let mut f = Fixture::new();
    f.cert();
    f.runner.set("signal", "security:import");
    f.runner.set("signal_number", &signal);
    let flag = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let _signals = macos::Signals::install(&flag).unwrap();
    assert!(f.package(false).is_err());
    assert!(
        f.runner
            .calls("security")
            .iter()
            .any(|a| a[0] == "delete-keychain")
    );
    assert!(!f.path("dist/.notarized").exists());
    let runner = SystemRunner::default().with_interruption(flag);
    assert_eq!(
        runner.interruption_code(),
        if signal == "INT" { 130 } else { 143 }
    );
    println!("signal cleanup verified");
    std::process::exit(runner.interruption_code() as i32);
}
#[test]
fn int_and_term_clean_keychain() {
    for (signal, code) in [("INT", 130), ("TERM", 143)] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "macos::tests::packaging::signal_helper",
                "--nocapture",
            ])
            .env("XTASK_TEST_SIGNAL", signal)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("signal cleanup verified"));
    }
}

#[test]
fn stable_versions_preserve_the_scripts_decimal_format_and_numeric_order() {
    for invalid in [
        "",
        "1.2",
        "1.2.3.4",
        "01.2.3",
        "1.2.3\n",
        "1.2.3+build",
        "1.2.3-rc.1",
        "-1.2.3",
    ] {
        assert!(macos::stable_version(invalid).is_err());
    }
    assert!(macos::stable_version("1.10.0").unwrap() > macos::stable_version("1.2.3").unwrap());
    assert!(
        macos::stable_version("18446744073709551616.0.0").unwrap()
            > macos::stable_version("999.0.0").unwrap()
    );
}
