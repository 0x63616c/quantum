//! Compile the production cache verbatim, adding private benchmark access only.
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "../voxelite-render/src/chunk_mesh.rs";
    println!("cargo:rerun-if-changed={source}");
    println!("cargo:rerun-if-changed=src/mesh_adapter.rs");
    println!("cargo:rustc-check-cfg=cfg(feature, values(\"gpu\"))");
    let original = fs::read_to_string(source)?;
    // The original module's tests require its renderer context. Our differential
    // tests exercise the copied production implementation alongside the queries.
    let (body, _) = original
        .split_once("#[cfg(test)]\nmod reference;")
        .ok_or("production chunk mesh test marker changed")?;
    let body = body.replacen("//!", "//", 1);
    let adapter = fs::read_to_string("src/mesh_adapter.rs")?;
    let output = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR missing")?);
    fs::write(output.join("production_mesh.rs"), body + &adapter)?;
    Ok(())
}
