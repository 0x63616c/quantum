//! Small launcher: builds the GPU harness in release and preserves its exit status.
use std::process::{Command, ExitCode};
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let subcommand = args.first().map_or("help", String::as_str);
    if ["help", "--help", "-h"].contains(&subcommand) {
        println!(
            "cargo xtask bench [--scene S ...] [--runs N] [--out FILE] [--compare [FILE]] [--update-baseline] [--markdown FILE]\n  [--memory-pct 10] [--rss-pct 50 --rss-mb 128]\ncargo xtask visual [--scene S ...] [--rebaseline] [--determinism] [--local-baseline]\n\nPass --help after bench or visual for all options. See docs/perf/rendering-harness.md."
        );
        return ExitCode::SUCCESS;
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
