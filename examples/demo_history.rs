//! Regenerate the bundled demo without a renderer or display.
//! cargo run --locked -p voxelite-format --example demo-history [-- --check]
#[path = "support/autumn_workshop.rs"]
mod autumn_workshop;

#[expect(clippy::expect_used, reason = "internal invariant: generated recipe")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/autumn-workshop.voxl");
    let check = match std::env::args().nth(1).as_deref() {
        None => false,
        Some("--check") => true,
        Some(_) => return Err("usage: demo-history [--check]".into()),
    };
    let project = autumn_workshop::build()?;
    let bytes = project.to_bytes()?;
    // Validate the serialized recipe, including replay, before replacing the asset.
    voxelite_format::from_bytes(&bytes)?;
    if check {
        if std::fs::read(&path)? != bytes {
            return Err("bundled demo differs; rerun without --check".into());
        }
    } else {
        std::fs::write(&path, &bytes)?;
    }
    eprintln!(
        "{} build steps; {} bytes; {}",
        project
            .build_history()
            .expect("generated recipe")
            .steps
            .len(),
        bytes.len(),
        path.display()
    );
    Ok(())
}
