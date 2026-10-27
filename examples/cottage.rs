//! Export the original MIT-licensed Autumn Workshop and its reproducible build history.
//! Run: cargo run --release -p voxelite-export --example cottage
#[path = "support/autumn_workshop.rs"]
mod autumn_workshop;
use std::path::Path;
use voxelite_render::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/screenshots".into());
    std::fs::create_dir_all(&output)?;
    let project = autumn_workshop::build()?;
    std::fs::write("examples/autumn-workshop.voxl", project.to_bytes()?)?;
    std::fs::write(
        Path::new(&output).join("autumn-workshop.glb"),
        voxelite_export::glb(&project.document.content)?,
    )?;
    for (name, direction, perspective, size) in [
        ("autumn-workshop", PresetView::Iso, false, 1024),
        ("autumn-workshop-perspective", PresetView::Iso, true, 768),
        ("autumn-workshop-front", PresetView::Front, false, 512),
        ("autumn-workshop-back", PresetView::Back, false, 512),
        ("autumn-workshop-top", PresetView::Top, false, 512),
    ] {
        let start = std::time::Instant::now();
        let options = RenderOptions {
            width: size,
            height: size,
            view: View::Preset {
                direction,
                perspective,
            },
            ..RenderOptions::default()
        };
        let image = voxelite_render::render(&project.document.content, &options)?;
        let png = voxelite_export::png_rgba(image.width, image.height, &image.rgba)?;
        std::fs::write(Path::new(&output).join(format!("{name}.png")), png)?;
        eprintln!("{name}: {:?}", start.elapsed());
    }
    eprintln!(
        "{} occupied layer-voxels; {} visible; semantic hash {}",
        project
            .document
            .content
            .layers
            .iter()
            .map(|l| l.voxels.len())
            .sum::<usize>(),
        project.document.content.composite().len(),
        project.semantic_hash()?
    );
    Ok(())
}
