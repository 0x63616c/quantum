//! Repository maintenance CLI and release-mode launcher for the GPU harness.

use std::{ffi::OsString, process::ExitCode};

use clap::{Args, CommandFactory, Parser, Subcommand};
use xtask::{
    check,
    cmd::{Command, Runner, SystemRunner},
    version,
    workspace::Workspace,
};

const HELP: &str = "{about}

Usage: {usage}

Checks:
  validate      Run all registered checks
  check         Run named checks, or --list
  tidy          Check tracked root entries, file sizes and junk files

Maintenance:
  bump-version  Update the stable workspace version and current references

Rendering:
  bench         Run performance harness (all arguments forwarded)
  visual        Run visual harness (all arguments forwarded)

Help:
  help          Print this help

Options:
{options}
Pass --help after a command for its options. Bench/visual show harness help.
";

#[derive(Parser)]
#[command(name = "cargo xtask", bin_name = "cargo xtask", about = "Repository checks, version maintenance and rendering harness", disable_help_subcommand = true, help_template = HELP)]
struct Cli {
    #[command(subcommand)]
    command: Option<Task>,
}

#[derive(Subcommand)]
enum Task {
    /// Run all registered checks (use --strict in CI)
    Validate(Validate),
    /// Run named checks or list the registry
    Check(Check),
    /// Check tracked root entries, file sizes and junk files
    Tidy,
    /// Update the stable workspace version and its current references
    BumpVersion {
        /// major, minor, patch or an explicit stable X.Y.Z
        version: String,
        /// Print planned changes without writing files or updating Cargo.lock
        #[arg(long)]
        dry_run: bool,
    },
    /// Run performance harness; forwards all arguments, including --help
    #[command(disable_help_flag = true)]
    Bench(Harness),
    /// Run visual harness; forwards all arguments, including --help
    #[command(disable_help_flag = true)]
    Visual(Harness),
    /// Print this help
    Help,
}

#[derive(Args)]
struct Check {
    /// Check names, in registry execution order
    #[arg(required_unless_present = "list", conflicts_with = "list")]
    names: Vec<String>,
    /// List registered checks without executing them
    #[arg(long)]
    list: bool,
    /// Fail when a required external tool is not installed
    #[arg(long)]
    strict: bool,
}

#[derive(Args)]
struct Validate {
    /// Omit a check (repeatable)
    #[arg(long, value_name = "NAME", num_args = 1..)]
    skip: Vec<String>,
    /// Run only these checks (repeatable)
    #[arg(long, value_name = "NAME", num_args = 1..)]
    only: Vec<String>,
    /// Fail when a required external tool is not installed
    #[arg(long)]
    strict: bool,
}

#[derive(Args)]
struct Harness {
    /// Arguments passed verbatim to voxelite-bench
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<OsString>,
}

fn main() -> ExitCode {
    let cli = parse_cli(std::env::args_os()).unwrap_or_else(|error| error.exit());
    match dispatch(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn parse_cli(args: impl IntoIterator<Item = OsString>) -> Result<Cli, clap::Error> {
    let args: Vec<_> = args.into_iter().collect();
    let mut cli = Cli::try_parse_from(&args)?;
    if let Some(Task::Bench(harness) | Task::Visual(harness)) = &mut cli.command {
        // Clap consumes an initial `--` delimiter. Preserve it too: the harness
        // owns interpretation of every argument after its subcommand.
        harness.args = args.into_iter().skip(2).collect();
    }
    Ok(cli)
}

fn dispatch(cli: Cli) -> Result<u8, Box<dyn std::error::Error>> {
    match cli.command {
        None | Some(Task::Help) => {
            Cli::command().print_help()?;
            println!();
        }
        Some(Task::Tidy) => xtask::tidy().map_err(xtask::Error::Invalid)?,
        Some(Task::Check(Check { list: true, .. })) => {
            for check in check::CHECKS {
                println!("{:<10} {}", check.name, check.description);
            }
        }
        Some(Task::Check(Check { names, strict, .. })) => return checks(&names, &[], strict),
        Some(Task::Validate(Validate { only, skip, strict })) => {
            return checks(&only, &skip, strict);
        }
        Some(Task::BumpVersion { version, dry_run }) => {
            let workspace = Workspace::discover()?;
            let runner = if dry_run {
                SystemRunner::dry_run()
            } else {
                SystemRunner::default()
            };
            let plan = version::Plan::prepare(&workspace, &runner, &version)?;
            plan.apply(&runner)?;
            if plan.changed_paths().is_empty() {
                println!("Workspace version is already {}; no changes.", plan.to());
            }
            for path in plan.changed_paths() {
                println!(
                    "{} {path}: {} -> {}",
                    if dry_run { "Would update" } else { "Updated" },
                    plan.from(),
                    plan.to()
                );
            }
        }
        Some(Task::Bench(harness)) => return Ok(run_harness("bench", &harness.args)),
        Some(Task::Visual(harness)) => return Ok(run_harness("visual", &harness.args)),
    }
    Ok(0)
}

fn checks(
    only: &[String],
    skip: &[String],
    strict: bool,
) -> Result<u8, Box<dyn std::error::Error>> {
    let selected = check::select(check::CHECKS, only, skip)?;
    let workspace = Workspace::discover()?;
    let context = check::Context {
        workspace: &workspace,
        runner: &SystemRunner::default(),
        strict,
    };
    let summary = check::run(&selected, &context, &mut std::io::stdout().lock())?;
    Ok(u8::from(!summary.success()))
}

fn harness_command(workspace: &Workspace, subcommand: &str, args: &[OsString]) -> Command {
    let mut command =
        workspace
            .cargo()
            .args(["run", "--release", "--locked", "-p", "voxelite-bench"]);
    if args
        .windows(2)
        .any(|pair| pair[0] == "--scene" && pair[1] == "terrain10m")
    {
        command = command.args(["--features", "large-scenes"]);
    }
    command.arg("--").arg(subcommand).args(args)
}

fn run_harness(subcommand: &str, args: &[OsString]) -> u8 {
    match Workspace::discover() {
        Ok(workspace) => harness_status(
            &harness_command(&workspace, subcommand, args),
            &SystemRunner::default(),
        ),
        Err(error) => {
            eprintln!("cannot run render harness: {error}");
            2
        }
    }
}

fn harness_status(command: &Command, runner: &dyn Runner) -> u8 {
    match command.status(runner) {
        Ok(status) => status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1),
        Err(error) => {
            eprintln!("cannot run render harness: {error}");
            2
        }
    }
}

#[cfg(test)]
mod tests;
