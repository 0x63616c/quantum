use super::*;
use crate::testing::RecordingRunner;

#[test]
fn builder_preserves_arguments_environment_and_directory() {
    let command = Command::new("tool")
        .arg("a b")
        .args(["--flag", "$(literal)"])
        .secret_arg("token")
        .env("VISIBLE", "a b")
        .secret_env("TOKEN", "private")
        .env("REMOVED", "old")
        .env_remove("REMOVED")
        .current_dir("/tmp/work space");
    assert_eq!(command.program(), "tool");
    assert_eq!(
        command.arguments().collect::<Vec<_>>(),
        ["a b", "--flag", "$(literal)", "token"]
    );
    assert_eq!(command.directory(), Some(Path::new("/tmp/work space")));
    assert_eq!(
        command.environment().collect::<Vec<_>>(),
        [
            (OsStr::new("REMOVED"), None),
            (OsStr::new("TOKEN"), Some(OsStr::new("private"))),
            (OsStr::new("VISIBLE"), Some(OsStr::new("a b"))),
        ]
    );
    for rendered in [command.to_string(), format!("{command:?}")] {
        assert!(!rendered.contains("token") && !rendered.contains("private"));
        assert!(rendered.contains("[REDACTED]") && rendered.contains("(unset REMOVED)"));
        assert!(rendered.contains("\"a b\""));
    }
}

#[test]
fn diagnostics_redact_repeated_secrets_before_quoting() {
    let secret = "private\n\"credential\"\\suffix";
    let command = Command::new(format!("tool-{secret}"))
        .secret_env("TOKEN", secret)
        .env("COPY", secret)
        .arg(format!("--token={secret}"))
        .current_dir(format!("/tmp/{secret}"));
    for rendered in [command.to_string(), format!("{command:?}")] {
        assert!(!rendered.contains("private"), "{rendered}");
        assert!(!rendered.contains("credential"), "{rendered}");
        assert!(rendered.contains("[REDACTED]"));
    }
}

#[test]
fn overlapping_stdin_and_argument_secrets_are_redacted_together() {
    for (argument, input) in [
        ("shared", "shared private remainder\n"),
        ("shared private remainder", "shared\n"),
    ] {
        let command = Command::new("tool")
            .secret_arg(argument)
            .secret_stdin(input.as_bytes())
            .arg(format!("copy={input}"));
        let runner = SystemRunner::with_dry_run_output(Output {
            status: Status::from_code(1),
            stderr: format!("{argument} {input}").into_bytes(),
            ..Output::default()
        });
        let error = command.output(&runner).unwrap_err();
        for message in [command.to_string(), error.to_string(), format!("{error:?}")] {
            assert!(!message.contains("shared"), "{message}");
            assert!(!message.contains("private remainder"), "{message}");
        }
    }
}

#[test]
fn capture_trim_and_exit_policy_are_explicit() {
    let command = Command::new("tool");
    let runner = RecordingRunner::stdout(b" \n answer\r\n");
    assert_eq!(command.read(&runner).unwrap(), "answer");
    assert!(runner.commands.borrow()[0].1);
    let runner = RecordingRunner::stdout(b"\xff\0");
    assert_eq!(command.output(&runner).unwrap().stdout, b"\xff\0");
    assert!(matches!(
        command.read(&RecordingRunner::stdout(b"\xff")),
        Err(Error::Utf8 { .. })
    ));
    for code in [0, 1, 17] {
        let runner = SystemRunner::with_dry_run_output(Output {
            status: Status::from_code(code),
            ..Output::default()
        });
        assert_eq!(command.status(&runner).unwrap().code(), Some(code));
        assert_eq!(command.run(&runner).is_ok(), code == 0);
        assert_eq!(command.output(&runner).is_ok(), code == 0);
    }
}

#[test]
fn errors_include_command_status_and_bounded_redacted_stderr() {
    let secret = "secret-value";
    let command = Command::new("sign tool")
        .arg("--key")
        .secret_arg(secret)
        .secret_env("TOKEN", "environment-secret");
    let runner = SystemRunner::with_dry_run_output(Output {
        status: Status::from_code(23),
        stderr: format!(
            "first line\n{}\nsecret-value environment-secret\nlast line",
            "é".repeat(5000)
        )
        .into_bytes(),
        ..Output::default()
    });
    let error = command.output(&runner).unwrap_err();
    for message in [error.to_string(), format!("{error:?}")] {
        assert!(message.contains("sign tool") && message.contains("23"));
        assert!(message.contains("last line") && message.contains("[REDACTED]"));
        assert!(!message.contains(secret) && !message.contains("environment-secret"));
        assert!(!message.contains("first line"));
        assert!(message.len() < 4500);
    }
    let runner = RecordingRunner::default();
    runner.results.borrow_mut().push_back(Err(io::Error::other(
        "failed secret-value environment-secret",
    )));
    let error = command.run(&runner).unwrap_err();
    assert!(matches!(error, Error::Spawn { .. }));
    assert!(!format!("{error:?}").contains(secret));
    assert_eq!(
        stderr_tail(&(0..30).map(|n| format!("line {n}\n")).collect::<String>())
            .lines()
            .count(),
        22
    );
}

#[test]
fn dry_runs_do_not_spawn_and_allow_read_only_opt_out() {
    let command = Command::new("this-executable-must-not-exist-xtask");
    let runner = SystemRunner::dry_run();
    assert!(runner.is_dry_run());
    command.run(&runner).unwrap();
    assert!(matches!(
        command.allow_in_dry_run().run(&runner),
        Err(Error::Spawn { .. })
    ));
    let runner = SystemRunner::with_dry_run_output(Output {
        stdout: b"planned\n".to_vec(),
        ..Output::default()
    });
    assert_eq!(Command::new("not-run").read(&runner).unwrap(), "planned");
    assert!(!SystemRunner::default().is_dry_run());
}
