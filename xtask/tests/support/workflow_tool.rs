// Standalone Rust fixture compiled by macos_workflows.rs. Never a production target.
#![forbid(unsafe_code)]
use std::{env, fs, io::Write, path::Path, process::exit};
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let executable = env::args().next().unwrap();
    let name = Path::new(&executable)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    if let Ok(log) = env::var("CALLS") {
        // Fixture arguments are ASCII. Rust's quoted vector representation is JSON.
        writeln!(
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log)
                .unwrap(),
            "{args:?}"
        )
        .unwrap();
    }
    match name {
        "cargo" if args[0] == "metadata" => println!(
            "{{\"packages\":[{{\"id\":\"app\",\"name\":\"voxelite-app\",\"version\":\"1.2.3\"}}],\"workspace_members\":[\"app\"]}}"
        ),
        "cargo" if args.iter().any(|a| a == "xtask") => {
            fs::create_dir_all("target/release").unwrap();
            // The binary is already built; never open an executable for writing
            // while parallel test threads may fork and inherit that descriptor.
            #[cfg(unix)]
            std::os::unix::fs::symlink(env::var("REAL_XTASK").unwrap(), "target/release/xtask")
                .unwrap();
            #[cfg(not(unix))]
            fs::copy(env::var("REAL_XTASK").unwrap(), "target/release/xtask").unwrap();
        }
        "cargo" => {
            fs::write(env::var("BUILT").unwrap(), "").unwrap();
        }
        "xtask" => {
            assert_eq!(args, ["macos", "package", "--skip-build"]);
            assert_eq!(env::var("VERSION").unwrap(), "1.2.3");
            fs::write("packaged", "").unwrap();
        }
        "gh" => {
            if args.get(1) == env::var("FAIL_AT").ok().as_ref() {
                exit(1);
            }
        }
        "pipe-parent" => {
            let mut child =
                std::process::Command::new(Path::new(&executable).with_file_name("pipe-leaf"))
                    .spawn()
                    .unwrap();
            if env::var_os("PIPE_PARENT_EXITS").is_none() {
                child.wait().unwrap();
            }
        }
        "pipe-leaf" => {
            #[cfg(unix)]
            {
                use std::{io::Read, os::unix::net::UnixStream};
                let mut ready = UnixStream::connect(env::var_os("PIPE_READY").unwrap()).unwrap();
                ready.write_all(&[1]).unwrap();
                // Only cancellation or test cleanup releases the inherited pipes.
                let mut byte = [0];
                assert_eq!(ready.read(&mut byte).unwrap(), 0);
            }
        }
        "secret-echo" => {
            for name in [
                "MACOS_CERTIFICATE_P12_BASE64",
                "MACOS_CERTIFICATE_PASSWORD",
                "MACOS_SIGNING_IDENTITY",
                "APPLE_API_KEY_P8_BASE64",
                "APPLE_API_KEY_ID",
                "APPLE_API_ISSUER_ID",
                "SPARKLE_ED_PRIVATE_KEY",
            ] {
                assert!(
                    env::var_os(name).is_none(),
                    "inherited credential variable {name}"
                );
            }
            let mut input = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut input).unwrap();
            assert_eq!(input, "fixture secret stdin\n");
            print!("{}", "x".repeat(128 * 1024));
            eprintln!("{input}");
            exit(17);
        }
        _ => panic!("unexpected fixture name {name}"),
    }
}
