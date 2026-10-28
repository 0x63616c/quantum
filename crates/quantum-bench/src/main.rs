//! Command-line entry point for the independent render checks.
use quantum_bench::{
    Result, bench,
    cli::{HELP, Options},
    compare, report, visual,
};
#[expect(clippy::print_stdout, reason = "CLI output")]
fn run() -> Result<bool> {
    let options = Options::parse(std::env::args().skip(1))?;
    match options.command.as_str() {
        "bench" => {
            let report = if let Some(path) = &options.input {
                report::read_json(path)?
            } else {
                bench::run(&options)?
            };
            if options.input.is_none() {
                report::write_json(&options.out, &report)?;
                println!("Benchmark JSON: {}", options.out.display());
            }
            let baseline = std::path::PathBuf::from("bench/baseline")
                .join(format!("{}.json", report.meta.machine_key));
            // Keep a usable report even when comparison refuses an incompatible or
            // missing baseline (especially useful for manual workflow artifacts).
            if let Some(path) = &options.markdown {
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                quantum_format::storage::safe_write(path, compare::summary(&report))?;
            }
            let (markdown, failed) = if let Some(compare) = &options.compare {
                let path = compare.as_ref().unwrap_or(&baseline);
                let old=report::read_json(path).map_err(|e|format!("{e}; create a same-machine baseline with cargo xtask bench --update-baseline"))?;
                compare::compare(&old, &report, (&options).into())?
            } else {
                (compare::summary(&report), false)
            };
            print!("{markdown}");
            if let Some(path) = &options.markdown {
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    std::fs::create_dir_all(parent)?;
                }
                quantum_format::storage::safe_write(path, &markdown)?;
            }
            if options.update_baseline {
                report::write_json(&baseline, &report)?;
                println!(
                    "Updated explicit performance baseline: {}",
                    baseline.display()
                );
            }
            Ok(!failed)
        }
        "visual" => visual::run(&options),
        _ => {
            println!("{HELP}");
            Ok(true)
        }
    }
}
fn main() {
    match run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("render harness: {error}");
            std::process::exit(2);
        }
    }
}
