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
        ["tidy", "fmt", "clippy", "test", "deny", "machete"]
    );
    assert_eq!(
        names(select(CHECKS, &["test".into(), "fmt".into(), "fmt".into()], &[]).unwrap()),
        ["fmt", "test"]
    );
    assert_eq!(
        names(select(CHECKS, &[], &["test".into()]).unwrap()),
        ["tidy", "fmt", "clippy", "deny", "machete"]
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
        let outcome = CHECKS[4].run(&context);
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
        &["test", "--workspace", "--locked"],
        &["deny", "check"],
        &["machete"],
    ];
    for (check, args) in CHECKS[1..].iter().zip(expected) {
        let runner =
            RecordingRunner::stdout(b"    fmt\n    clippy\n    test\n    deny\n    machete\n");
        runner.results.borrow_mut().push_back(Ok(Output::default()));
        let context = Context {
            workspace: &workspace,
            runner: &runner,
            strict: false,
        };
        assert_eq!(check.run(&context), Outcome::Pass);
        runner.assert_args(0, &["--list"]);
        runner.assert_args(1, args);
        assert!(!runner.commands.borrow()[1].1);
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
        matches!(CHECKS[4].run(&context), Outcome::Fail(messages) if messages[0].contains("17"))
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
    assert!(matches!(CHECKS[4].run(&context), Outcome::Fail(_)));
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
