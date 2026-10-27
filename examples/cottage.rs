//! Export the original MIT-licensed Autumn Workshop and its reproducible build history.
//! Run: cargo run --release -p voxelite-plugin-exporters --example cottage
#[path = "support/autumn_workshop.rs"]
mod autumn_workshop;
use std::path::Path;
use voxelite_render::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "docs/screenshots".into());
    if output == "--ao-comparisons" {
        return ao_comparisons();
    }
    std::fs::create_dir_all(&output)?;
    let project = autumn_workshop::build()?;
    std::fs::write("examples/autumn-workshop.voxl", project.to_bytes()?)?;
    std::fs::write(
        Path::new(&output).join("autumn-workshop.glb"),
        voxelite_plugin_exporters::encoding::glb(&project.document.content)?,
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
        let png =
            voxelite_plugin_exporters::encoding::png_rgba(image.width, image.height, &image.rgba)?;
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

// Render checked-in models at one fixed camera/size per pair. The single voxel
// is an unoccluded control: changing AO must not change its lighting.
fn ao_comparisons() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "docs/screenshots/ao".into());
    std::fs::create_dir_all(&output)?;
    let autumn =
        voxelite_format::from_bytes(include_bytes!("../bench/scenes/autumn-workshop.voxl"))?
            .document
            .content;
    let step = serde_json::from_slice(include_bytes!(
        "../crates/plugins/voxelite-plugin-exporters/tests/golden/scene.json"
    ))?;
    let voxel = voxelite_format::from_bytes(include_bytes!(
        "../spec/voxl/1.0/fixtures/populated.canonical.voxl"
    ))?
    .document
    .content;
    for (name, snapshot) in [
        ("autumn-workshop", autumn),
        ("step", step),
        ("single-voxel", voxel),
    ] {
        let camera = Camera::fit(&snapshot, PresetView::Iso, 1.0, false)?;
        for (suffix, strength) in [
            ("off", AmbientOcclusionStrength::OFF),
            ("default", AmbientOcclusionStrength::default()),
        ] {
            let image = render(
                &snapshot,
                &RenderOptions {
                    width: 384,
                    height: 384,
                    view: View::Camera(camera),
                    background: [235, 235, 230, 255],
                    ambient_occlusion_strength: strength,
                    ..Default::default()
                },
            )?;
            let png = voxelite_plugin_exporters::encoding::png_rgba(
                image.width,
                image.height,
                &image.rgba,
            )?;
            assert!(png.len() < 400_000, "keep documentation images small");
            let path = Path::new(&output).join(format!("{name}-{suffix}.png"));
            std::fs::write(&path, &png)?;
            eprintln!("{}: {} bytes", path.display(), png.len());
        }
    }
    Ok(())
}
