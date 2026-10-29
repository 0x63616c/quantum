//! Validate a project and write canonical bytes to a different path.
use std::{fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: roundtrip INPUT.voxl OUTPUT.voxl (output must not exist)".into());
    }
    let project = quantum_format::read(BufReader::new(File::open(&args[0])?))?;
    // Validate and encode before creating the destination; never overwrite input.
    let bytes = project.to_bytes()?;
    let mut output = File::create_new(&args[1])?;
    std::io::Write::write_all(&mut output, &bytes)?;
    Ok(())
}
