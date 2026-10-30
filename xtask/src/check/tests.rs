use super::*;
use crate::{
    cmd::{Output, Status},
    testing::RecordingRunner,
};

#[test]
fn selection_validates_names_and_preserves_order() {
    let names = |selected: Vec<&Check>| selected.iter().map(|c| c.name).collect::<Vec<_>>();
    assert_eq!(
        names(select(CHECKS, &[], &[]).unwrap()),
        [
            "boundaries",
            "source-lints",
            "tidy",
            "workflows",
            "plugin-deps",
            "brand",
            "docs-links",
            "docs-assets",
            "appcast",
            "icons",
            "fmt",
            "clippy",
            "test",
            "deny",
            "machete"
        ]
    );
    assert_eq!(
        names(select(CHECKS, &["test".into(), "fmt".into(), "fmt".into()], &[]).unwrap()),
        ["fmt", "test"]
    );
    assert_eq!(
        names(select(CHECKS, &[], &["test".into()]).unwrap()),
        [
            "boundaries",
            "source-lints",
            "tidy",
            "workflows",
            "plugin-deps",
            "brand",
            "docs-links",
            "docs-assets",
            "appcast",
            "icons",
            "fmt",
            "clippy",
            "deny",
            "machete"
        ]
    );
    assert!(select(CHECKS, &["typo".into()], &[]).is_err());
    assert!(select(CHECKS, &[], &["typo".into()]).is_err());
    assert!(select(CHECKS, &["fmt".into()], &["fmt".into()]).is_err());
    assert!(select(&[], &[], &[]).is_err());
}

#[test]
fn missing_tools_skip_or_fail_without_running_the_check() {
    let workspace = Workspace::discover().unwrap();
    for strict in [false, true] {
        let runner = RecordingRunner::stdout(b"Installed Commands:\n    test Run tests\n");
        let context = Context {
            workspace: &workspace,
            runner: &runner,
            strict,
        };
        let outcome = CHECKS
            .iter()
            .find(|check| check.name == "deny")
            .unwrap()
            .run(&context);
        let message = "cargo deny is not installed; install it with `cargo install cargo-deny`";
        assert_eq!(
            outcome,
            if strict {
                Outcome::Fail(vec![message.into()])
            } else {
                Outcome::Skip(message.into())
            }
        );
        assert_eq!(runner.commands.borrow().len(), 1);
    }
}

#[test]
fn missing_rustup_components_skip_or_fail_despite_listed_proxies() {
    let workspace = Workspace::discover().unwrap();
    for (tool, component) in [("fmt", "rustfmt"), ("clippy", "clippy-driver")] {
        for executable in [format!("cargo-{tool}"), component.into()] {
            for strict in [false, true] {
                let runner = RecordingRunner::stdout(b"    fmt\n    clippy\n");
                runner.results.borrow_mut().push_back(Ok(Output {
                    status: Status::from_code(1),
                    stderr: format!(
                        "error: '{executable}' is not installed for the toolchain 'stable'.\n"
                    )
                    .into_bytes(),
                    ..Output::default()
                }));
                let context = Context {
                    workspace: &workspace,
                    runner: &runner,
                    strict,
                };
                let component = if tool == "fmt" { "rustfmt" } else { "clippy" };
                let reason = format!(
                    "cargo {tool} is not installed; install it with `rustup component add {component}`"
                );
                assert_eq!(
                    by_name(tool).run(&context),
                    if strict {
                        Outcome::Fail(vec![reason])
                    } else {
                        Outcome::Skip(reason)
                    }
                );
                runner.assert_args(1, &[tool, "--version"]);
                assert_eq!(runner.commands.borrow().len(), 2);
            }
        }
    }
}

#[test]
fn broken_component_probes_fail_without_running_the_gate() {
    let workspace = Workspace::discover().unwrap();
    let runner = RecordingRunner::stdout(b"    clippy\n");
    runner.results.borrow_mut().push_back(Ok(Output {
        status: Status::from_code(17),
        stderr: b"toolchain installation is corrupt\n".to_vec(),
        ..Output::default()
    }));
    let context = Context {
        workspace: &workspace,
        runner: &runner,
        strict: false,
    };
    assert!(
        matches!(by_name("clippy").run(&context), Outcome::Fail(messages)
        if messages[0].contains("17") && messages[0].contains("corrupt"))
    );
    assert_eq!(runner.commands.borrow().len(), 2);
}

#[test]
fn wrappers_keep_exact_gates_and_inherit_stdio() {
    let workspace = Workspace::discover().unwrap();
    let expected: &[&[&str]] = &[
        &["fmt", "--all", "--", "--check"],
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        &["deny", "check"],
        &["machete"],
    ];
    for args in expected {
        let check = CHECKS.iter().find(|check| check.name == args[0]).unwrap();
        let runner =
            RecordingRunner::stdout(b"    fmt\n    clippy\n    test\n    deny\n    machete\n");
        let probes_component = matches!(args[0], "fmt" | "clippy");
        if probes_component {
            runner.results.borrow_mut().push_back(Ok(Output::default()));
        }
        runner.results.borrow_mut().push_back(Ok(Output::default()));
        let context = Context {
            workspace: &workspace,
            runner: &runner,
            strict: false,
        };
        assert_eq!(check.run(&context), Outcome::Pass);
        runner.assert_args(0, &["--list"]);
        let gate = if probes_component {
            runner.assert_args(1, &[args[0], "--version"]);
            assert!(runner.commands.borrow()[1].1);
            2
        } else {
            1
        };
        runner.assert_args(gate, args);
        assert!(!runner.commands.borrow()[gate].1);
    }
}

#[test]
fn installed_but_broken_tools_fail_even_without_strict() {
    let workspace = Workspace::discover().unwrap();
    let runner = RecordingRunner::stdout(b"    deny\n");
    runner.results.borrow_mut().push_back(Ok(Output {
        status: Status::from_code(17),
        ..Output::default()
    }));
    let context = Context {
        workspace: &workspace,
        runner: &runner,
        strict: false,
    };
    assert!(
        matches!(CHECKS.iter().find(|check| check.name == "deny").unwrap().run(&context), Outcome::Fail(messages) if messages[0].contains("17"))
    );
    let runner = RecordingRunner::default();
    runner
        .results
        .borrow_mut()
        .push_back(Err(io::Error::other("cargo unavailable")));
    let context = Context {
        workspace: &workspace,
        runner: &runner,
        strict: false,
    };
    assert!(matches!(
        CHECKS
            .iter()
            .find(|check| check.name == "deny")
            .unwrap()
            .run(&context),
        Outcome::Fail(_)
    ));
}

#[test]
fn report_continues_after_failure_and_counts_skips_separately() {
    let checks = [
        Check::new("bad", "bad", |_| {
            Outcome::Fail(vec!["one\nproblem".into(), "two".into()])
        }),
        Check::new("absent", "absent", |_| Outcome::Skip("tool missing".into())),
        Check::new("good", "good", |_| Outcome::Pass),
    ];
    let workspace = Workspace::discover().unwrap();
    let runner = RecordingRunner::default();
    let context = Context {
        workspace: &workspace,
        runner: &runner,
        strict: false,
    };
    let mut report = Vec::new();
    let summary = run(&checks.iter().collect::<Vec<_>>(), &context, &mut report).unwrap();
    assert_eq!(
        summary,
        Summary {
            passed: 1,
            failed: 1,
            skipped: 1
        }
    );
    assert!(!summary.success());
    let report = String::from_utf8(report).unwrap();
    assert_eq!(report.lines().count(), 4);
    assert!(report.starts_with("FAIL bad ("));
    assert!(report.contains("one problem; two"));
    assert!(report.contains("SKIP absent (") && report.contains("PASS good ("));
    assert!(report.contains("1 passed, 1 failed, 1 skipped ("));
}

fn by_name(name: &str) -> &'static Check {
    CHECKS.iter().find(|check| check.name == name).unwrap()
}

#[test]
fn lint_selection_covers_all_policies_without_duplicate_clippy() {
    let names = LINT_CHECKS
        .iter()
        .map(|name| (*name).into())
        .collect::<Vec<_>>();
    assert_eq!(
        select(CHECKS, &names, &[])
            .unwrap()
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>(),
        LINT_CHECKS
    );
}

#[test]
fn workspace_tests_fall_back_when_nextest_is_absent() {
    let workspace = Workspace::discover().unwrap();
    let runner = RecordingRunner::stdout(b"    test\n");
    runner.results.borrow_mut().extend([
        Ok(Output {
            stdout: b"    test\n".to_vec(),
            ..Output::default()
        }),
        Ok(Output::default()),
    ]);
    let context = Context {
        workspace: &workspace,
        runner: &runner,
        strict: true,
    };
    assert_eq!(by_name("test").run(&context), Outcome::Pass);
    runner.assert_args(2, &["test", "--workspace", "--locked"]);
    assert_eq!(runner.commands.borrow().len(), 3);
}

#[test]
fn nextest_failures_do_not_skip_doctests_or_fall_back() {
    let workspace = Workspace::discover().unwrap();
    for (test_code, doc_code) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        let runner = RecordingRunner::stdout(b"    nextest\n    test\n");
        runner.results.borrow_mut().extend([
            Ok(Output {
                stdout: b"    nextest\n".to_vec(),
                ..Output::default()
            }),
            Ok(Output {
                status: Status::from_code(test_code),
                ..Output::default()
            }),
            Ok(Output {
                stdout: b"    test\n".to_vec(),
                ..Output::default()
            }),
            Ok(Output {
                status: Status::from_code(doc_code),
                ..Output::default()
            }),
        ]);
        let context = Context {
            workspace: &workspace,
            runner: &runner,
            strict: false,
        };
        let outcome = by_name("test").run(&context);
        if test_code == 0 && doc_code == 0 {
            assert_eq!(outcome, Outcome::Pass);
        } else {
            assert!(
                matches!(outcome, Outcome::Fail(errors) if errors.len() == test_code as usize + doc_code as usize)
            );
        }
        runner.assert_args(
            2,
            &[
                "nextest",
                "run",
                "--workspace",
                "--locked",
                "--profile",
                "ci",
            ],
        );
        runner.assert_args(4, &["test", "--doc", "--workspace", "--locked"]);
        assert_eq!(runner.commands.borrow().len(), 5);
    }
}
