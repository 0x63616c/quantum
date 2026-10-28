use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn policy() -> Redactor {
    Redactor::new(Some("/private/custom-root")).unwrap()
}
fn email() -> String {
    ["PRIVATE", "example.invalid"].join("@")
}
fn environment() -> Environment {
    Environment {
        app_version: "0.3.1".into(),
        os_name: "TestOS".into(),
        os_version: "1".into(),
        arch: "test-arch".into(),
    }
}
fn feedback() -> Report {
    Report::Feedback(FeedbackReport {
        environment: environment(),
        what_happened: "Something happened".into(),
        kind: FeedbackKind::Bug,
        contact_email: None,
    })
}
fn crash() -> Report {
    Report::Crash(CrashReport {
        environment: environment(),
        panic_message: "Test panic".into(),
        location: Some("source.rs:1:2".into()),
        backtrace: "frame".into(),
        timestamp_utc: 0,
        log_lines: vec!["recent".into()],
        user_note: None,
        contact_email: None,
        scene_attachment: false,
    })
}

#[test]
fn redaction_edges() {
    let r = policy();
    for root in [
        "/Users/PRIVATE",
        "/home/PRIVATE",
        "/home/隐私",
        "/Users/PRIVATE NAME",
    ] {
        assert_eq!(r.redact(&format!("({root}/file.rs:12)")), "(~/file.rs:12)");
        assert_eq!(r.redact(root), "~");
    }
    assert_eq!(
        r.redact("/private/custom-root/a /private/custom-root-other/a"),
        "~/a /private/custom-root-other/a"
    );
    assert_eq!(r.redact("/private/custom-root /private/custom-root"), "~ ~");
    assert_eq!(r.redact("/Users/ /home/"), "/Users/ /home/");
    assert_eq!(
        r.redact(&format!("<{}>, {}", email(), email())),
        "<[redacted email]>, [redacted email]"
    );
    assert_eq!(
        r.redact(&["隐私+tag", "example.invalid"].join("@")),
        "[redacted email]"
    );
    for local in ["PRIVATE+tag", "PRIVATE'part", "\"PRIVATE TEXT\""] {
        assert_eq!(
            r.redact(&[local, "example.invalid"].join("@")),
            "[redacted email]"
        );
    }
    assert_eq!(
        r.redact(&["PRIVATE", "[127.0.0.1]"].join("@")),
        "[redacted email]"
    );
    assert_eq!(r.redact("plain text, no address"), "plain text, no address");
    assert_eq!(
        Redactor::new(Some("/")).unwrap().redact("/source/file"),
        "/source/file"
    );
    assert_eq!(r.redact(&r.redact("/home/PRIVATE/file")), "~/file");
}

#[test]
fn every_free_text_field_is_redacted_and_contact_is_opt_in() {
    let mut report = crash();
    if let Report::Crash(r) = &mut report {
        let private = format!("/home/PRIVATE/file {}", email());
        r.environment = Environment {
            app_version: private.clone(),
            os_name: private.clone(),
            os_version: private.clone(),
            arch: private.clone(),
        };
        r.panic_message = private.clone();
        r.location = Some(private.clone());
        r.backtrace = private.clone();
        r.log_lines = vec![private.clone()];
        r.user_note = Some(private);
    }
    let clean = policy().report(&report).unwrap();
    let json = serde_json::to_string(&clean).unwrap();
    assert!(!json.contains("PRIVATE"));
    assert_eq!(json.matches("[redacted email]").count(), 9);
    if let Report::Crash(r) = &mut report {
        r.contact_email = Some(email());
    }
    let clean = policy().report(&report).unwrap();
    if let Report::Crash(r) = clean {
        assert_eq!(r.contact_email, Some(email()));
    }
    assert!(report.markdown(&policy()).unwrap().contains(&email()));
}

#[test]
fn feedback_and_persisted_data_share_privacy_policy() {
    let mut report = feedback();
    if let Report::Feedback(r) = &mut report {
        r.what_happened = format!("/Users/PRIVATE NAME/source {}", email());
        r.contact_email = Some(format!("/private/custom-root/contact {}", email()));
    }
    let payload = report
        .payload(&format!("Report {}", email()), &policy())
        .unwrap();
    assert!(!payload.title.contains(&email()));
    assert!(!payload.body.contains("PRIVATE"));
    assert!(!payload.body.contains("/private/custom-root"));
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), &report, &policy()).unwrap();
    let json = std::fs::read_to_string(&path).unwrap();
    assert!(!json.contains("PRIVATE"));
    assert!(!json.contains(&email()));
    assert!(!json.contains("/private/custom-root"));
    assert_eq!(
        load(&path, &policy()).unwrap(),
        policy().report(&report).unwrap()
    );
}

#[test]
fn feedback_markdown_is_stable_and_payload_matches_preview() {
    let report = feedback();
    let expected = "## What is sent\n\nThis preview becomes a public GitHub issue via the configured relay or browser.\nIncludes version/OS/architecture and the details below. Contact opted in: false.\nScene consent: false; scene contents are never included by this crate.\nHome paths and free-text emails are redacted. Review all text before confirming.\n\n## Environment\n\n> Quantum 0.3.1\n> TestOS 1\n> test-arch\n\n## What happened\n\n> Something happened\n\n## Contact\n\n> Not provided\n";
    assert_eq!(report.markdown(&policy()).unwrap(), expected);
    let payload = report.payload("Bug", &policy()).unwrap();
    assert_eq!(payload.body, expected);
    assert_eq!(payload.schema_version, 1);
    assert_eq!(payload.labels, ["type:bug"]);
    let value = serde_json::to_value(&payload).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 5);
    assert_eq!(
        serde_json::from_value::<RelayPayload>(value).unwrap(),
        payload
    );
}

#[test]
fn crash_sections_and_untrusted_markdown() {
    let mut report = crash();
    if let Report::Crash(r) = &mut report {
        r.panic_message = "</details>\n## Spoof\n```".into();
        r.scene_attachment = true;
    }
    let body = report.markdown(&policy()).unwrap();
    for section in [
        "## Panic",
        "## Location",
        "## Backtrace",
        "## Recent logs",
        "## User note",
        "## Timestamp (Unix seconds, UTC)",
    ] {
        assert!(body.contains(section));
    }
    assert!(body.contains("> &lt;/details&gt;\n> ## Spoof\n> ```"));
    assert!(body.contains("Scene consent: true; scene contents are never included"));
}

#[test]
fn fallback_encodes_every_query_and_caps_final_bytes() {
    let mut report = feedback();
    if let Report::Feedback(r) = &mut report {
        r.kind = FeedbackKind::Feature;
        r.what_happened = "Unicode 雪 & ? # = +\nnew line".into();
    }
    let payload = report.payload("雪 & ? # = +", &policy()).unwrap();
    let url = payload.github_url(100_000).unwrap();
    let parsed = url::Url::parse(&url).unwrap();
    let pairs: std::collections::HashMap<_, _> = parsed.query_pairs().collect();
    assert_eq!(pairs["body"], payload.body);
    assert_eq!(pairs["title"], payload.title);
    assert_eq!(pairs["template"], "feature.yml");
    assert_eq!(pairs["labels"], "type:feat");
    assert!(parsed.fragment().is_none());
    assert_eq!(payload.github_url(url.len()).unwrap(), url);
    assert!(matches!(
        payload.github_url(url.len() - 1),
        Err(Error::UrlTooLong)
    ));
    assert!(matches!(payload.github_url(0), Err(Error::UrlTooLong)));
    assert!(
        feedback()
            .payload("Bug", &policy())
            .unwrap()
            .github_url(100_000)
            .unwrap()
            .contains("template=bug.yml")
    );
}

#[test]
fn requests_are_explicit_and_status_failures_do_not_delete_reports() {
    struct Transport {
        calls: AtomicUsize,
        status: u16,
    }
    impl RelayTransport for Transport {
        type Error = ();
        fn post(&self, _: &RelayRequest) -> Result<u16, ()> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.status)
        }
    }
    let t = Transport {
        calls: AtomicUsize::new(0),
        status: 201,
    };
    let payload = feedback().payload("Bug", &policy()).unwrap();
    let request = payload
        .request("https://relay.invalid/reports", "0.3.1")
        .unwrap();
    assert_eq!(
        request.headers,
        [
            ("Content-Type".into(), "application/json".into()),
            ("User-Agent".into(), "Quantum/0.3.1".into())
        ]
    );
    assert_eq!(
        serde_json::from_str::<RelayPayload>(&request.json_body).unwrap(),
        payload
    );
    assert_eq!(t.calls.load(Ordering::SeqCst), 0);
    request.send(&t).unwrap();
    assert_eq!(t.calls.load(Ordering::SeqCst), 1);
    let dir = tempfile::tempdir().unwrap();
    let path = write(dir.path(), &crash(), &policy()).unwrap();
    for status in [199, 300, 400, 429, 500] {
        assert!(
            matches!(request.send(&Transport { calls: AtomicUsize::new(0), status }), Err(SendError::Status(s)) if s == status)
        );
        assert!(path.exists());
    }
    struct Failed;
    impl RelayTransport for Failed {
        type Error = ();
        fn post(&self, _: &RelayRequest) -> Result<u16, ()> {
            Err(())
        }
    }
    assert!(matches!(
        request.send(&Failed),
        Err(SendError::Transport(()))
    ));
    for url in [
        "",
        "http://relay.invalid",
        "https://relay.invalid/#fragment",
        "https://relay.invalid/?token=secret",
        "file:///tmp/relay",
        "https://user:pass@relay.invalid",
    ] {
        assert!(payload.request(url, "0.3.1").is_err());
    }
    for version in [
        "",
        "0.3.1\r\nInjected: value",
        "private@example.invalid",
        "0.3.1/private",
    ] {
        assert!(payload.request("https://relay.invalid", version).is_err());
    }
}

#[test]
fn storage_roundtrip_pending_dismiss_and_failures() {
    let dir = tempfile::tempdir().unwrap();
    assert!(list_pending(&dir.path().join("absent")).unwrap().is_empty());
    let first = write(dir.path(), &crash(), &policy()).unwrap();
    let second = write(dir.path(), &feedback(), &policy()).unwrap();
    assert_ne!(first, second);
    assert_eq!(load(&first, &policy()).unwrap(), crash());
    std::fs::write(dir.path().join("crash-incomplete.tmp"), "partial").unwrap();
    std::fs::write(dir.path().join("other.json"), "{}").unwrap();
    std::fs::create_dir(dir.path().join("crash-directory.json")).unwrap();
    assert_eq!(list_pending(dir.path()).unwrap().len(), 2);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&first).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    mark_sent(&first).unwrap();
    dismiss(&second).unwrap();
    dismiss(&second).unwrap();
    assert!(list_pending(dir.path()).unwrap().is_empty());
    assert!(load(&first, &policy()).is_err());
    let invalid = dir.path().join("invalid");
    std::fs::write(&invalid, "invalid JSON").unwrap();
    assert!(load(&invalid, &policy()).is_err());
    assert!(write(&invalid, &crash(), &policy()).is_err());
    let mut large = feedback();
    if let Report::Feedback(r) = &mut large {
        r.what_happened = "x".repeat(4 * 1024 * 1024 + 1);
    }
    assert!(write(dir.path(), &large, &policy()).is_err());
    std::fs::write(&invalid, "x".repeat(4 * 1024 * 1024 + 1)).unwrap();
    assert!(load(&invalid, &policy()).is_err());
}

#[test]
fn panic_hook_writes_and_chains_even_on_io_failure() {
    // A child process isolates process-global hooks from the parallel test runner.
    const CHILD: &str = "QUANTUM_CRASH_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::panic_hook_writes_and_chains_even_on_io_failure",
            ])
            .env(CHILD, "1")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    static CHAINED: AtomicUsize = AtomicUsize::new(0);
    std::panic::set_hook(Box::new(|_| {
        CHAINED.fetch_add(1, Ordering::SeqCst);
    }));
    let dir = tempfile::tempdir().unwrap();
    let directory = dir.path().join("pending");
    install_panic_hook(HookConfig {
        directory: directory.clone(),
        environment: environment(),
        redactor: policy(),
        log_lines: vec!["old".into(), "/home/PRIVATE/recent".into()],
        max_log_lines: 1,
    })
    .unwrap();
    assert!(std::panic::catch_unwind(|| panic!("forced diagnostic test")).is_err());
    let pending = list_pending(&directory).unwrap();
    assert_eq!(pending.len(), 1);
    let Report::Crash(report) = load(&pending[0], &policy()).unwrap() else {
        panic!("wrong report type")
    };
    assert_eq!(report.panic_message, "forced diagnostic test");
    assert_eq!(report.log_lines, ["~/recent"]);
    assert!(report.location.unwrap().contains("tests.rs:"));
    assert!(!report.backtrace.is_empty());
    assert!(report.timestamp_utc > 0);
    assert_eq!(report.contact_email, None);
    assert!(!report.scene_attachment);
    dismiss(&pending[0]).unwrap();
    std::fs::remove_dir(&directory).unwrap();
    std::fs::write(&directory, "block directory").unwrap();
    assert!(std::panic::catch_unwind(|| panic!("forced IO failure test")).is_err());
    assert_eq!(CHAINED.load(Ordering::SeqCst), 2);
}
