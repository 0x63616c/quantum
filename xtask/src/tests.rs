use super::*;
use xtask::cmd::{Output, Status};

#[test]
fn cli_definition_is_consistent() {
    let mut command = Cli::command();
    command.clone().debug_assert();
    let help = command.render_help().to_string();
    for subcommand in command.get_subcommands() {
        assert!(
            help.lines()
                .any(|line| line.split_whitespace().next() == Some(subcommand.get_name())),
            "missing command in grouped help: {}",
            subcommand.get_name()
        );
    }
    for heading in ["Checks:", "Maintenance:", "Rendering:"] {
        assert!(help.contains(heading));
    }
}

#[test]
fn harness_arguments_and_help_are_forwarded_verbatim() {
    for subcommand in ["bench", "visual"] {
        for args in [
            vec!["--help"],
            vec!["-h"],
            vec![
                "--scene",
                "autumn",
                "--scene",
                "terrain10m",
                "--runs",
                "3",
                "--compare",
                "--markdown",
                "path with spaces.md",
            ],
            vec!["--input", "$(literal)", "--unknown-future-flag", "-1"],
            vec!["--", "--scene", "terrain10m"],
            vec!["--scene", "autumn", "--", "--help"],
            vec![],
        ] {
            let raw = ["xtask", subcommand]
                .into_iter()
                .chain(args.iter().copied());
            let cli = parse_cli(raw.map(OsString::from)).unwrap();
            let Some(Task::Bench(harness) | Task::Visual(harness)) = cli.command else {
                panic!("expected harness")
            };
            assert_eq!(harness.args, args, "{subcommand}");
            let workspace = Workspace::discover().unwrap();
            let command = harness_command(&workspace, subcommand, &harness.args);
            let mut expected = vec!["run", "--release", "--locked", "-p", "voxelite-bench"];
            if args
                .windows(2)
                .any(|pair| pair == ["--scene", "terrain10m"])
            {
                expected.extend(["--features", "large-scenes"]);
            }
            expected.extend(["--", subcommand]);
            expected.extend(args);
            assert_eq!(command.arguments().collect::<Vec<_>>(), expected);
            assert_eq!(command.directory(), Some(workspace.root()));
        }
    }
}

#[cfg(unix)]
#[test]
fn harness_accepts_non_utf8_paths() {
    use std::os::unix::ffi::OsStringExt;
    let path = OsString::from_vec(b"path-\xff".to_vec());
    let cli = parse_cli([
        "xtask".into(),
        "bench".into(),
        "--input".into(),
        path.clone(),
    ])
    .unwrap();
    let Some(Task::Bench(harness)) = cli.command else {
        panic!("expected bench")
    };
    assert_eq!(harness.args, [OsString::from("--input"), path]);
}

#[test]
fn checks_require_names_or_list_and_validate_accepts_filters() {
    for args in [
        vec!["xtask", "check"],
        vec!["xtask", "check", "--list", "fmt"],
        vec!["xtask", "tidy", "extra"],
        vec!["xtask", "unknown"],
    ] {
        assert_eq!(Cli::try_parse_from(args).err().unwrap().exit_code(), 2);
    }
    assert!(matches!(
        Cli::try_parse_from(["xtask", "check", "--list"])
            .unwrap()
            .command,
        Some(Task::Check(Check { list: true, .. }))
    ));
    let cli = Cli::try_parse_from([
        "xtask", "validate", "--only", "fmt", "tidy", "--only", "deny", "--skip", "test",
        "--strict",
    ])
    .unwrap();
    let Some(Task::Validate(validate)) = cli.command else {
        panic!("expected validate")
    };
    assert_eq!(validate.only, ["fmt", "tidy", "deny"]);
    assert_eq!(validate.skip, ["test"]);
    assert!(validate.strict);
}

#[test]
fn harness_propagates_exit_codes_and_spawn_failure_is_two() {
    for (code, expected) in [(0, 0), (1, 1), (2, 2), (42, 42), (255, 255), (256, 1)] {
        let runner = SystemRunner::with_dry_run_output(Output {
            status: Status::from_code(code),
            ..Output::default()
        });
        assert_eq!(
            harness_status(&Command::new("not-executed"), &runner),
            expected
        );
    }
    assert_eq!(
        harness_status(
            &Command::new("xtask-nonexistent-tool"),
            &SystemRunner::default()
        ),
        2
    );
}

#[cfg(unix)]
#[test]
fn harness_signal_termination_is_failure() {
    use std::os::unix::process::ExitStatusExt;
    let runner = SystemRunner::with_dry_run_output(Output {
        status: std::process::ExitStatus::from_raw(9).into(),
        ..Output::default()
    });
    assert_eq!(harness_status(&Command::new("not-executed"), &runner), 1);
}

#[test]
fn screenshots_ci_arguments_are_forwarded() {
    for args in [
        vec!["--help"],
        vec![
            "--remote",
            "origin",
            "--base",
            "base-ref",
            "--head",
            "HEAD",
            "--publish",
            "154",
        ],
        vec!["--screen", "welcome", "--out", "path with spaces"],
    ] {
        let cli = parse_cli(
            ["xtask", "screenshots"]
                .into_iter()
                .chain(args.iter().copied())
                .map(OsString::from),
        )
        .unwrap();
        let Some(Task::Screenshots(task)) = cli.command else {
            panic!("expected screenshots")
        };
        assert_eq!(task.args, args);
    }
}

#[test]
fn lint_command_parses_without_optional_or_skip_flags() {
    assert!(matches!(
        parse_cli(["xtask", "lint"].map(OsString::from))
            .unwrap()
            .command,
        Some(Task::Lint)
    ));
    assert!(parse_cli(["xtask", "lint", "--skip", "clippy"].map(OsString::from)).is_err());
}
