use super::support::*;
use crate::macos::{self, appcast, files, validate, validate_release};
use serde_json::json;
use std::{collections::BTreeMap, fs};

fn release(version: &str, draft: bool, prerelease: bool, feed: bool) -> serde_json::Value {
    let mut assets = vec![
        json!({"name":"Quantum.dmg", "size":11, "browser_download_url":format!("{}v{version}/Quantum.dmg", macos::DOWNLOAD)}),
        json!({"name":"Quantum.html", "size":13, "browser_download_url":format!("{}v{version}/Quantum.html", macos::DOWNLOAD)}),
    ];
    if feed {
        assets.push(json!({"name":"appcast.xml", "size":900, "browser_download_url":format!("{}v{version}/appcast.xml", macos::DOWNLOAD)}));
    }
    json!({"tag_name":format!("v{version}"), "draft":draft, "prerelease":prerelease, "assets":assets})
}
#[test]
fn empty_feed_is_explicit_and_has_no_installable_item() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../site/appcast.xml");
    assert!(validate(&path, true, None).unwrap().is_empty());
    assert!(validate(&path, false, None).is_err());
}
#[test]
fn offline_validation_matches_actual_files_and_history() {
    let f = Fixture::new();
    f.package(false).unwrap();
    write(
        f.path("dist/previous-appcast.xml"),
        fixture_feed(&["1.2.2"]),
    );
    write(
        f.path("site/appcast.xml"),
        fixture_feed(&["1.2.3", "1.2.2"]),
    );
    write(f.path("dist/Quantum.html"), "fixture notes");
    let check = || {
        validate_release(
            &f.path("site/appcast.xml"),
            &f.path("dist"),
            &f.path("dist/previous-appcast.xml"),
        )
    };
    check().unwrap();
    let path = f.path("dist/Quantum.app/Contents/Info.plist");
    let original = files::read_plist(&path).unwrap();
    for key in [
        "CFBundleVersion",
        "CFBundleShortVersionString",
        "LSMinimumSystemVersion",
    ] {
        let mut info = original.clone();
        info.insert(key.into(), "9.9.9".into());
        plist::Value::Dictionary(info).to_file_xml(&path).unwrap();
        assert!(check().is_err());
    }
    plist::Value::Dictionary(original)
        .to_file_xml(&path)
        .unwrap();
    for name in ["dist/Quantum.dmg", "dist/Quantum.html"] {
        let old = fs::read(f.path(name)).unwrap();
        let mut altered = old.clone();
        altered.push(b'!');
        write(f.path(name), altered);
        assert!(check().is_err());
        write(f.path(name), old);
    }
    write(f.path("site/appcast.xml"), fixture_feed(&["1.2.3"]));
    assert!(check().is_err());
}
#[test]
fn url_signature_length_and_metadata_rejection() {
    let f = Fixture::new();
    let base = fixture_feed(&["1.2.3"]);
    let url = format!("{}v1.2.3/Quantum.dmg", macos::DOWNLOAD);
    for (old, new) in [
        (
            url.as_str(),
            "https://github.com.evil.invalid/0x63616c/quantum/releases/download/v1.2.3/Quantum.dmg",
        ),
        (
            url.as_str(),
            "https://github.com/0x63616c/quantum/releases/download/v1.2.2/Quantum.dmg",
        ),
        (
            url.as_str(),
            "https://github.com/0x63616c/quantum/releases/download/v1.2.3/../Quantum.dmg",
        ),
        (
            url.as_str(),
            "https://github.com/0x63616c/quantum/releases/download/v1.2.3/Quantum.dmg?redirect=1",
        ),
        (SIGNATURE, ""),
        (SIGNATURE, "not base64!"),
        (SIGNATURE, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
        ("length=\"11\"", "length=\"0\""),
        ("length=\"11\"", "length=\"-1\""),
        ("length=\"11\"", "length=\"1.1\""),
    ] {
        write(f.path("feed.xml"), base.replacen(old, new, 1));
        assert!(validate(&f.path("feed.xml"), false, None).is_err(), "{new}");
    }
    // Specifically exercise the release-notes signature (the second occurrence).
    write(
        f.path("feed.xml"),
        base.replace(
            &format!("s:length=\"13\" s:edSignature=\"{SIGNATURE}\""),
            "s:length=\"13\" s:edSignature=\"\"",
        ),
    );
    assert!(validate(&f.path("feed.xml"), false, None).is_err());
    for tag in [
        "enclosure",
        "s:minimumSystemVersion",
        "s:shortVersionString",
        "s:releaseNotesLink",
    ] {
        let start = base.find(&format!("<{tag}")).unwrap();
        let end = if tag == "enclosure" {
            base[start..].find("/>").unwrap() + start + 2
        } else {
            base.find(&format!("</{tag}>")).unwrap() + tag.len() + 3
        };
        let mut modified = base.clone();
        modified.replace_range(start..end, "");
        write(f.path("feed.xml"), modified);
        assert!(validate(&f.path("feed.xml"), false, None).is_err(), "{tag}");
    }
    let extra = base.replace("</item>", "<s:deltas><enclosure/></s:deltas></item>");
    write(f.path("feed.xml"), extra);
    assert!(validate(&f.path("feed.xml"), false, None).is_err());
}
#[test]
fn malformed_xml_and_unstable_or_unordered_versions() {
    let f = Fixture::new();
    for data in [
        "<rss>",
        "<html/>",
        "<!DOCTYPE rss [<!ENTITY x \"x\">]><rss/>",
        "<rss version=\"2.0\"><channel/><channel/></rss>",
    ] {
        write(f.path("feed.xml"), data);
        assert!(validate(&f.path("feed.xml"), false, None).is_err());
    }
    for versions in [
        vec!["1.2.3-rc.1"],
        vec!["01.2.3"],
        vec!["1.2.3", "1.2.3"],
        vec!["1.2.3", "1.10.0"],
    ] {
        write(f.path("feed.xml"), fixture_feed(&versions));
        assert!(validate(&f.path("feed.xml"), false, None).is_err());
    }
}

#[test]
fn rss_elements_require_the_original_unnamespaced_tags() {
    let f = Fixture::new();
    for data in [
        "<rss xmlns='urn:wrong' version='2.0'><channel/></rss>",
        "<x:rss xmlns:x='urn:wrong' version='2.0'><channel/></x:rss>",
        "<rss version='2.0'><channel xmlns='urn:wrong'/></rss>",
    ] {
        write(f.path("feed.xml"), data);
        assert!(validate(&f.path("feed.xml"), true, None).is_err(), "{data}");
    }
    let feed = fixture_feed(&["1.2.3"]);
    let namespaced_item = feed
        .replace("<item>", "<x:item xmlns:x='urn:wrong'>")
        .replace("</item>", "</x:item>");
    write(f.path("feed.xml"), namespaced_item);
    assert!(validate(&f.path("feed.xml"), false, None).is_err());
    // ElementTree's .//enclosure only counts unqualified enclosure tags.
    write(
        f.path("feed.xml"),
        feed.replace("</item>", "<x:enclosure xmlns:x='urn:other'/></item>"),
    );
    validate(&f.path("feed.xml"), false, None).unwrap();
}

#[test]
fn appcast_dry_run_preserves_existing_outputs_without_release_artifacts() {
    let mut f = Fixture::new();
    f.env.set("SPARKLE_ED_PRIVATE_KEY", SECRET_KEY);
    f.env.set(
        "GITHUB_OUTPUT",
        f.path("outputs").to_str().unwrap().to_owned(),
    );
    for path in ["site/appcast.xml", "outputs", "dist/sentinel"] {
        write(f.path(path), "keep existing bytes");
    }
    appcast::generate(&macos::Context {
        root: f.root(),
        runner: &crate::cmd::SystemRunner::dry_run(),
        environment: &f.env,
    })
    .unwrap();
    for path in ["site/appcast.xml", "outputs", "dist/sentinel"] {
        assert_eq!(
            fs::read_to_string(f.path(path)).unwrap(),
            "keep existing bytes"
        );
    }
    assert!(!f.path("dist/Quantum.html").exists());
    assert_eq!(fs::read_dir(f.path("tmp")).unwrap().count(), 0);
}
#[test]
fn asset_metadata_must_cover_every_asset() {
    let f = Fixture::new();
    write(f.path("feed.xml"), fixture_feed(&["1.2.3"]));
    let mut sizes: BTreeMap<_, _> = [
        (format!("{}v1.2.3/Quantum.dmg", macos::DOWNLOAD), 11),
        (format!("{}v1.2.3/Quantum.html", macos::DOWNLOAD), 13),
    ]
    .into();
    validate(&f.path("feed.xml"), false, Some(&sizes)).unwrap();
    assert!(validate(&f.path("feed.xml"), false, Some(&BTreeMap::new())).is_err());
    sizes.insert(format!("{}v1.2.3/Quantum.dmg", macos::DOWNLOAD), 12);
    assert!(validate(&f.path("feed.xml"), false, Some(&sizes)).is_err());
}
#[test]
fn paginated_releases_select_numeric_highest_feed_and_copy_verbatim() {
    let f = Fixture::new();
    let feed = format!(
        "{}<!-- signed exact bytes -->\n",
        fixture_feed(&["1.10.0", "1.2.3"])
    );
    f.runner.set("feed", &feed);
    f.runner.set(
        "releases",
        &json!([
            [
                release("1.2.3", false, false, true),
                release("3.0.0", true, false, true)
            ],
            [
                release("1.10.0", false, false, true),
                release("4.0.0", false, true, true),
                release("2.0.0", false, false, false),
                release("9.0.0-rc.1", false, false, true)
            ]
        ])
        .to_string(),
    );
    appcast::fetch(&f.context(), &f.path("output/feed.xml")).unwrap();
    assert_eq!(fs::read_to_string(f.path("output/feed.xml")).unwrap(), feed);
}
#[test]
fn no_releases_or_no_feed_uses_empty_feed() {
    for releases in [
        json!([]),
        json!([[]]),
        json!([[release("1.0.0", false, false, false)]]),
    ] {
        let f = Fixture::new();
        f.runner.set("releases", &releases.to_string());
        appcast::fetch(&f.context(), &f.path("output/feed.xml")).unwrap();
        assert!(
            validate(&f.path("output/feed.xml"), true, None)
                .unwrap()
                .is_empty()
        );
        write(f.path("site/appcast.xml"), fixture_feed(&["1.2.3"]));
        assert!(appcast::fetch(&f.context(), &f.path("output/feed.xml")).is_err());
    }
}
#[test]
fn api_download_and_invalid_asset_errors_preserve_destination() {
    let f = Fixture::new();
    f.runner.set("feed", &fixture_feed(&["1.10.0", "1.2.3"]));
    let mut releases = json!([[
        release("1.10.0", false, false, true),
        release("1.2.3", false, false, true)
    ]]);
    f.runner.set("releases", &releases.to_string());
    write(f.path("output/feed.xml"), "previous deployment");
    for failure in ["gh:api", "gh:release", ""] {
        f.runner.set("fail", failure);
        if failure.is_empty() {
            releases[0][0]["assets"][0]["size"] = json!(99);
            f.runner.set("releases", &releases.to_string());
        }
        assert!(appcast::fetch(&f.context(), &f.path("output/feed.xml")).is_err());
        assert_eq!(
            fs::read_to_string(f.path("output/feed.xml")).unwrap(),
            "previous deployment"
        );
    }
}
#[test]
fn appcast_requires_credentials_and_notarization() {
    let mut f = Fixture::new();
    f.package(false).unwrap();
    assert_eq!(
        f.generate().unwrap_err().to_string(),
        "SPARKLE_ED_PRIVATE_KEY is required for a release appcast."
    );
    f.env.set("SPARKLE_ED_PRIVATE_KEY", SECRET_KEY);
    assert_eq!(
        f.generate().unwrap_err().to_string(),
        "A notarized DMG is required for a release appcast."
    );
}
#[test]
fn sparkle_signing_and_feed_validation() {
    let mut f = Fixture::new();
    f.prepare_feed();
    for (flag, value, success) in [
        ("", "", true),
        ("TAG", "v9.9.9", false),
        ("unsigned_feed", "1", false),
        ("wrong_url", "1", false),
        ("unsigned_metadata", "1", false),
        ("fail", "dist/sparkle/bin/generate_appcast", false),
        ("missing_notes", "1", false),
        ("wrong_notes", "1", false),
    ] {
        f.runner.flags.borrow_mut().clear();
        f.env.set("TAG", "v1.2.3");
        if flag == "TAG" {
            f.env.set(flag, value);
        } else if !flag.is_empty() {
            f.runner.set(flag, value);
        }
        fs::remove_file(f.path("outputs")).ok();
        let result = f.generate();
        assert_eq!(result.is_ok(), success, "{flag}: {result:?}");
        assert_eq!(
            fs::read_to_string(f.path("outputs"))
                .unwrap()
                .contains("generated=true"),
            success
        );
        if let Err(error) = result {
            assert!(
                !error.to_string().contains(DIAGNOSTIC) && !error.to_string().contains(SECRET_KEY)
            );
        }
        if success {
            let html = fs::read_to_string(f.path("dist/Quantum.html")).unwrap();
            // Independent expected bytes, including the wrapper and all five
            // html.escape substitutions; do not compare the encoder to itself.
            assert_eq!(
                html,
                concat!(
                    "<!doctype html><html><head><meta charset=\"utf-8\"></head><body>\n",
                    "<pre style=\"white-space: pre-wrap; font-family: system-ui\">",
                    "Fix &lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp; café &lt;/pre&gt; ]]&gt; &#x27;quoted&#x27;\n",
                    "</pre>\n</body></html>\n",
                )
            );
            assert!(
                html.contains("&lt;script&gt;")
                    && html.contains("&amp;")
                    && html.contains("café")
                    && html.contains("&#x27;quoted&#x27;")
            );
            assert!(!html.contains("<script>") && !html.contains("]]>"));
            let xml = fs::read_to_string(f.path("site/appcast.xml")).unwrap();
            assert!(xml.ends_with("<!-- fixture feed signature -->"));
        }
    }
    f.runner.flags.borrow_mut().clear();
    fs::copy(
        f.path("site/appcast.xml"),
        f.path("dist/previous-appcast.xml"),
    )
    .unwrap();
    let path = f.path("dist/Quantum.app/Contents/Info.plist");
    let mut info = files::read_plist(&path).unwrap();
    for key in ["CFBundleVersion", "CFBundleShortVersionString"] {
        info.insert(key.into(), "1.2.4".into());
    }
    plist::Value::Dictionary(info).to_file_xml(&path).unwrap();
    f.env.set("TAG", "v1.2.4");
    f.runner.set("drop_history", "1");
    assert!(f.generate().is_err());
    f.runner.flags.borrow_mut().clear();
    f.generate().unwrap();
    assert_eq!(
        validate(&f.path("site/appcast.xml"), false, None)
            .unwrap()
            .iter()
            .map(|i| i.version.as_str())
            .collect::<Vec<_>>(),
        ["1.2.4", "1.2.3"]
    );
    f.env.set("TAG", "v1.2.3");
    assert!(f.generate().is_err());
    f.env.set("TAG", "v1.2.4");
    let notes = fs::read(f.path("dist/RELEASE_NOTES.md")).unwrap();
    fs::remove_file(f.path("dist/RELEASE_NOTES.md")).unwrap();
    assert!(f.generate().is_err());
    write(f.path("dist/RELEASE_NOTES.md"), notes);
    write(f.path("dist/Quantum.dmg"), "corrupt dmg");
    assert!(f.generate().unwrap_err().to_string().contains("checksum"));
}
#[test]
fn missing_history_mismatched_repository_and_nonincreasing_versions_fail() {
    let mut f = Fixture::new();
    f.prepare_feed();
    for tag in ["v01.2.3", "v1.2.3-rc.1", "v1.2.3\nmain"] {
        f.env.set("TAG", tag);
        assert!(f.generate().is_err());
    }
    f.env.set("TAG", "v1.2.3");
    f.env.set("GITHUB_REPOSITORY", "wrong/repository");
    assert!(f.generate().is_err());
    f.env.set("GITHUB_REPOSITORY", macos::REPOSITORY);
    for version in ["1.2.3", "1.2.4"] {
        write(
            f.path("dist/previous-appcast.xml"),
            fixture_feed(&[version]),
        );
        assert!(f.generate().unwrap_err().to_string().contains("newer"));
    }
    fs::remove_file(f.path("dist/previous-appcast.xml")).unwrap();
    assert!(f.generate().is_err());
}

#[test]
fn legacy_signed_urls_validate_before_and_after_repository_redirects() {
    let f = Fixture::new();
    let legacy = fixture_feed(&["0.3.1"])
        .replace(macos::DOWNLOAD, macos::LEGACY_DOWNLOAD)
        .replace("Quantum.dmg", "Voxelite.dmg")
        .replace("Quantum.html", "Voxelite.html");
    let path = f.path("old.xml");
    write(&path, &legacy);
    let sizes = BTreeMap::from([
        (format!("{}v0.3.1/Voxelite.dmg", macos::DOWNLOAD), 11),
        (format!("{}v0.3.1/Voxelite.html", macos::DOWNLOAD), 13),
    ]);
    assert_eq!(
        validate(&path, false, Some(&sizes)).unwrap()[0].version,
        "0.3.1"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        legacy,
        "signed bytes must not change"
    );
    for impostor in [
        "https://github.com.evil.invalid/",
        "https://github.com/other/",
    ] {
        write(&path, legacy.replace("https://github.com/", impostor));
        assert!(validate(&path, false, None).is_err());
    }
}

#[test]
fn release_commands_require_the_canonical_repository() {
    let mut env = macos::Environment::default();
    env.set("GITHUB_REPOSITORY", macos::REPOSITORY);
    assert_eq!(macos::release_repository(&env).unwrap(), macos::REPOSITORY);
    env.set("GITHUB_REPOSITORY", macos::LEGACY_REPOSITORY);
    assert!(macos::release_repository(&env).is_err());
    env.set("GITHUB_REPOSITORY", "another-owner/quantum");
    assert!(macos::release_repository(&env).is_err());
}
