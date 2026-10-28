//! Ordered checks with explicit pass, failure and skip outcomes.

use std::{
    io::{self, Write},
    time::Instant,
};

use crate::{
    Error, Result, checks,
    cmd::{self, Runner},
    tidy,
    workspace::Workspace,
};

/// Result of one check. Skips are visible and do not count as passes.
#[derive(Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The check completed successfully.
    Pass,
    /// The check found one or more problems.
    Fail(Vec<String>),
    /// The check could not run for the stated reason.
    Skip(String),
}

/// Shared services and missing-tool policy for checks.
pub struct Context<'a> {
    /// Repository being checked.
    pub workspace: &'a Workspace,
    /// Process executor, replaceable in tests.
    pub runner: &'a dyn Runner,
    /// Treat missing tools as failures instead of skips.
    pub strict: bool,
}

enum Action {
    Function(fn(&Context<'_>) -> Outcome),
    Cargo(&'static [&'static str]),
}

/// One named check. Add a constructor call to [`CHECKS`] to register another.
pub struct Check {
    /// Stable name used by `check`, `--only` and `--skip`.
    pub name: &'static str,
    /// Short description shown by `check --list`.
    pub description: &'static str,
    action: Action,
}

impl Check {
    /// Register an in-process check.
    pub const fn new(
        name: &'static str,
        description: &'static str,
        run: fn(&Context<'_>) -> Outcome,
    ) -> Self {
        Self {
            name,
            description,
            action: Action::Function(run),
        }
    }

    /// Register a Cargo command; the first argument identifies its required tool.
    pub const fn cargo(
        name: &'static str,
        description: &'static str,
        args: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            description,
            action: Action::Cargo(args),
        }
    }

    fn run(&self, context: &Context<'_>) -> Outcome {
        match self.action {
            Action::Function(run) => run(context),
            Action::Cargo(args) => run_cargo(context, args),
        }
    }
}

/// Checks selected by the strict `cargo xtask lint` entry point.
pub const LINT_CHECKS: &[&str] = &["boundaries", "source-lints", "clippy"];

/// Registered checks in execution order: cheap local checks before builds and audits.
pub static CHECKS: &[Check] = &[
    Check::new(
        "boundaries",
        "Portable crate transitive dependency boundaries",
        checks::boundaries::run,
    ),
    Check::new(
        "source-lints",
        "Styling, string dispatch and source size ratchets",
        checks::source_lints::run,
    ),
    Check::new(
        "tidy",
        "Tracked root entries, size limits and junk files",
        tidy_check,
    ),
    Check::new(
        "workflows",
        "SHA-pinned workflow actions and explicit default permissions",
        checks::workflows::run,
    ),
    Check::new(
        "plugin-deps",
        "Plugin and MCP dependency boundaries",
        checks::plugin_deps::run,
    ),
    Check::new(
        "brand",
        "Branding inventory matches source and packaging",
        checks::brand::run,
    ),
    Check::new("typos", "Spelling in source and documentation", typos_check),
    Check::new(
        "docs-links",
        "Local links in tracked docs Markdown",
        checks::docs::links,
    ),
    Check::new(
        "docs-assets",
        "Documentation images and capture ignore policy",
        checks::docs::assets,
    ),
    Check::new(
        "appcast",
        "Offline stable feed validation (empty initial feed allowed)",
        appcast_check,
    ),
    Check::new("icons", "Reproducible Quantize brand assets", icons_check),
    Check::cargo("fmt", "Rust formatting", &["fmt", "--all", "--", "--check"]),
    Check::cargo(
        "clippy",
        "Workspace lint checks, including all targets",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
    ),
    Check::new("test", "Workspace tests and doctests", workspace_tests),
    Check::cargo(
        "deny",
        "Dependency licenses, advisories and sources",
        &["deny", "check"],
    ),
    Check::cargo("machete", "Unused dependencies", &["machete"]),
];

fn typos_check(context: &Context<'_>) -> Outcome {
    let sh = match xshell::Shell::new() {
        Ok(sh) => sh,
        Err(error) => return Outcome::Fail(vec![error.to_string()]),
    };
    let command = cmd::Command::from(xshell::cmd!(sh, "typos --config _typos.toml --hidden"))
        .current_dir(context.workspace.root());
    match command.run(context.runner) {
        Ok(()) => Outcome::Pass,
        Err(error) => Outcome::Fail(vec![format!(
            "typos: {error}; install typos-cli 1.51.1 with cargo install typos-cli --version 1.51.1 --locked"
        )]),
    }
}

fn appcast_check(context: &Context<'_>) -> Outcome {
    match crate::macos::validate(&context.workspace.path("site/appcast.xml"), true, None) {
        Ok(_) => Outcome::Pass,
        Err(error) => Outcome::Fail(vec![format!("Invalid appcast: {error}")]),
    }
}

fn icons_check(context: &Context<'_>) -> Outcome {
    match crate::icons::generate(context.workspace, true, None) {
        Ok(()) => Outcome::Pass,
        Err(error) => Outcome::Fail(vec![error.to_string()]),
    }
}

fn tidy_check(context: &Context<'_>) -> Outcome {
    match tidy::check(context.workspace, context.runner) {
        Ok(()) => Outcome::Pass,
        Err(error) => Outcome::Fail(error.lines().map(str::to_owned).collect()),
    }
}

fn workspace_tests(context: &Context<'_>) -> Outcome {
    let available = match context
        .workspace
        .cargo()
        .arg("--list")
        .allow_in_dry_run()
        .read(context.runner)
    {
        Ok(available) => available,
        Err(error) => return Outcome::Fail(vec![error.to_string()]),
    };
    if !available
        .lines()
        .any(|line| line.split_whitespace().next() == Some("nextest"))
    {
        return run_cargo(context, &["test", "--workspace", "--locked"]);
    }
    // Run doctests even if ordinary tests fail: nextest does not execute them.
    let tests = run_cargo(
        context,
        &[
            "nextest",
            "run",
            "--workspace",
            "--locked",
            "--profile",
            "ci",
        ],
    );
    let docs = run_cargo(context, &["test", "--doc", "--workspace", "--locked"]);
    match (tests, docs) {
        (Outcome::Pass, Outcome::Pass) => Outcome::Pass,
        (Outcome::Fail(mut tests), Outcome::Fail(docs)) => {
            tests.extend(docs);
            Outcome::Fail(tests)
        }
        (Outcome::Fail(errors), _) | (_, Outcome::Fail(errors)) => Outcome::Fail(errors),
        (Outcome::Skip(reason), _) | (_, Outcome::Skip(reason)) => Outcome::Skip(reason),
    }
}

fn run_cargo(context: &Context<'_>, args: &[&str]) -> Outcome {
    let Some(tool) = args.first() else {
        return Outcome::Fail(vec!["Cargo check has no subcommand".into()]);
    };
    // Ask Cargo so its selected toolchain and executable search rules are respected.
    let available = match context
        .workspace
        .cargo()
        .arg("--list")
        .allow_in_dry_run()
        .read(context.runner)
    {
        Ok(available) => available,
        Err(error) => return Outcome::Fail(vec![error.to_string()]),
    };
    if !available
        .lines()
        .any(|line| line.split_whitespace().next() == Some(tool))
    {
        return missing_tool(context, tool);
    }
    // Cargo lists rustup proxies even when their component is absent from the
    // selected toolchain. Probe those tools before running a workspace gate.
    if matches!(*tool, "fmt" | "clippy") {
        let probe = context
            .workspace
            .cargo()
            .args([tool, "--version"])
            .allow_in_dry_run()
            .output(context.runner);
        if let Err(error) = probe {
            if let cmd::Error::Failed { stderr, .. } = &error {
                let component = if *tool == "fmt" {
                    "rustfmt"
                } else {
                    "clippy-driver"
                };
                if [format!("cargo-{tool}"), component.into()]
                    .iter()
                    .any(|name| {
                        stderr.contains(&format!(
                            "error: '{name}' is not installed for the toolchain '"
                        ))
                    })
                {
                    return missing_tool(context, tool);
                }
            }
            return Outcome::Fail(vec![error.to_string()]);
        }
    }
    match context.workspace.cargo().args(args).run(context.runner) {
        Ok(()) => Outcome::Pass,
        Err(error) => Outcome::Fail(vec![error.to_string()]),
    }
}

fn missing_tool(context: &Context<'_>, tool: &str) -> Outcome {
    let installation = match tool {
        "fmt" => "rustup component add rustfmt".into(),
        "clippy" => "rustup component add clippy".into(),
        _ => format!("cargo install cargo-{tool}"),
    };
    let reason = format!("cargo {tool} is not installed; install it with `{installation}`");
    if context.strict {
        Outcome::Fail(vec![reason])
    } else {
        Outcome::Skip(reason)
    }
}

/// Select checks without changing registry order. Unknown or conflicting names fail
/// before any check executes. Empty `only` means every registered check.
pub fn select<'a>(checks: &'a [Check], only: &[String], skip: &[String]) -> Result<Vec<&'a Check>> {
    for name in only.iter().chain(skip) {
        if !checks.iter().any(|check| check.name == name) {
            return Err(Error::Invalid(format!(
                "unknown check {name:?}; use cargo xtask check --list"
            )));
        }
    }
    if let Some(name) = only.iter().find(|name| skip.contains(name)) {
        return Err(Error::Invalid(format!(
            "check {name:?} appears in both --only and --skip"
        )));
    }
    let selected: Vec<_> = checks
        .iter()
        .filter(|check| {
            (only.is_empty() || only.iter().any(|name| name == check.name))
                && !skip.iter().any(|name| name == check.name)
        })
        .collect();
    if selected.is_empty() {
        return Err(Error::Invalid("no checks selected".into()));
    }
    Ok(selected)
}

/// Counts from a complete check run.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Summary {
    /// Checks which passed.
    pub passed: usize,
    /// Checks which failed.
    pub failed: usize,
    /// Checks explicitly skipped because a tool was missing.
    pub skipped: usize,
}

impl Summary {
    /// Whether the run has no failures. Skips remain visible in the report.
    pub fn success(&self) -> bool {
        self.failed == 0
    }
}

/// Run every selected check, even after failures. Child output stays live; each
/// result occupies one timed line, followed by a summary. Writer errors propagate.
pub fn run(
    checks: &[&Check],
    context: &Context<'_>,
    report: &mut dyn Write,
) -> io::Result<Summary> {
    let start = Instant::now();
    let mut summary = Summary::default();
    for check in checks {
        let check_start = Instant::now();
        let (label, details) = match check.run(context) {
            Outcome::Pass => {
                summary.passed += 1;
                ("PASS", String::new())
            }
            Outcome::Fail(messages) => {
                summary.failed += 1;
                ("FAIL", messages.join("; "))
            }
            Outcome::Skip(reason) => {
                summary.skipped += 1;
                ("SKIP", reason)
            }
        };
        let details = details.split_whitespace().collect::<Vec<_>>().join(" ");
        let suffix = if details.is_empty() {
            String::new()
        } else {
            format!(" — {details}")
        };
        writeln!(
            report,
            "{label} {} ({:.2}s){suffix}",
            check.name,
            check_start.elapsed().as_secs_f64()
        )?;
        report.flush()?;
    }
    writeln!(
        report,
        "{} passed, {} failed, {} skipped ({:.2}s)",
        summary.passed,
        summary.failed,
        summary.skipped,
        start.elapsed().as_secs_f64()
    )?;
    Ok(summary)
}

#[cfg(test)]
mod tests;
