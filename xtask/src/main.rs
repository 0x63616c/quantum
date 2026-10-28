//! Repository hygiene checks and a release-mode launcher for the GPU harness.
use std::process::{Command, ExitCode};

mod screenshots;
mod tidy;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = args.first().map_or("help", String::as_str);
    if ["help", "--help", "-h"].contains(&subcommand) {
        println!(
            "cargo xtask bench [--scene S ...] [--runs N] [--out FILE] [--compare [FILE]] [--update-baseline] [--markdown FILE]\n  [--memory-pct 10] [--rss-pct 50 --rss-mb 128]\ncargo xtask visual [--scene S ...] [--rebaseline] [--determinism] [--local-baseline]\ncargo xtask tidy  Check tracked root entries, file sizes and junk files.\ncargo xtask screenshots [--base REF] [--head REF] [--publish PR]  Before/after captures of key screens for UI PRs.\n\nPass --help after bench or visual for all options. See docs/perf/rendering-harness.md."
        );
        return ExitCode::SUCCESS;
    }
    if subcommand == "tidy" {
        if args.len() != 1 {
            eprintln!("usage: cargo xtask tidy");
            return ExitCode::from(2);
        }
        return match tidy::run() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }
    if subcommand == "screenshots" {
        if args.get(1).is_some_and(|a| a == "--help" || a == "-h") {
            println!("{}", screenshots::USAGE);
            return ExitCode::SUCCESS;
        }
        return match screenshots::run(&args[1..]) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }
    if !["bench", "visual"].contains(&subcommand) {
        eprintln!("unknown xtask {subcommand:?}; use cargo xtask help");
        return ExitCode::from(2);
    }
    let mut command = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
    command.args(["run", "--release", "--locked", "-p", "voxelite-bench"]);
    if args
        .windows(2)
        .any(|v| v[0] == "--scene" && v[1] == "terrain10m")
    {
        command.args(["--features", "large-scenes"]);
    }
    command.arg("--").args(&args);
    match command.status() {
        Ok(status) => ExitCode::from(
            status
                .code()
                .and_then(|n| u8::try_from(n).ok())
                .unwrap_or(1),
        ),
        Err(error) => {
            eprintln!("cannot run render harness: {error}");
            ExitCode::from(2)
        }
    }
}
