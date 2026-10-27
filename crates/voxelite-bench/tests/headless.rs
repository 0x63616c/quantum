//! Opt-in smoke test: default workspace tests never require a GPU adapter.
use voxelite_bench::{block_on, style};
use voxelite_core::{Document, Uuid};
use voxelite_render::{
    Camera, PresetView,
    gpu::{Headless, HeadlessOptions},
};
#[test]
#[ignore = "requires a Metal or Vulkan adapter (lavapipe is supported)"]
fn tiny_real_viewport_capture_and_timestamps() {
    let snapshot = Document::new(Uuid::from_u128(1), Uuid::from_u128(2)).content;
    let camera = Camera::fit(&snapshot, PresetView::Iso, 1.0, false).unwrap();
    let mut gpu = block_on(Headless::new(HeadlessOptions {
        profile: true,
        ..Default::default()
    }))
    .unwrap();
    let frame = block_on(gpu.render(
        Some(snapshot.clone()),
        snapshot.bounds,
        camera,
        [17, 13],
        style(snapshot.bounds),
        true,
    ))
    .unwrap();
    assert_eq!(frame.size, [34, 26]);
    assert_eq!(frame.rgba.as_ref().unwrap().len(), 34 * 26 * 4);
    assert_eq!(frame.stats.target_size, [34, 26]);
    assert!(frame.stats.wall_ms > 0.0);
    let next = block_on(gpu.render(
        None,
        snapshot.bounds,
        camera,
        [17, 13],
        style(snapshot.bounds),
        true,
    ))
    .unwrap();
    assert_eq!(frame.rgba, next.rgba);
    assert_eq!(next.stats.chunks_updated, 0);
}

#[test]
#[ignore = "requires a Metal or Vulkan adapter (lavapipe is supported)"]
fn ambient_occlusion_changes_live_without_remeshing() {
    use voxelite_render::AmbientOcclusionStrength;
    let snapshot = voxelite_bench::scenes::load("autumn").unwrap();
    let bounds = snapshot.bounds;
    let camera = Camera::fit(&snapshot, PresetView::Iso, 1.0, false).unwrap();
    let mut gpu = block_on(Headless::new(HeadlessOptions::default())).unwrap();
    let mut style = style(bounds);
    let original = block_on(gpu.render(Some(snapshot), bounds, camera, [192, 192], style, true))
        .unwrap()
        .rgba
        .unwrap();
    for strength in [0.0, 0.5, 2.0, 1.0] {
        style.ambient_occlusion_strength = AmbientOcclusionStrength::new(strength).unwrap();
        let frame = block_on(gpu.render(None, bounds, camera, [192, 192], style, true)).unwrap();
        assert_eq!(frame.stats.chunks_updated, 0);
        let pixels = frame.rgba.unwrap();
        if strength == 1.0 {
            assert_eq!(
                pixels, original,
                "restoring default strength restores every pixel"
            );
        } else {
            assert_ne!(pixels, original, "strength must reach the shader");
        }
    }
}

#[test]
#[ignore = "renders autumn twice on a real adapter and writes diagnostic PNGs"]
fn single_voxel_recolor_exceeds_default_visual_tolerance() {
    use voxelite_bench::{paths, scenes, visual};
    use voxelite_core::{Actor, Command, Engine, Mirror, Mode, Origin, Transaction, Voxel};
    let snapshot = scenes::load("autumn").unwrap();
    let camera = paths::fit(&snapshot, visual::DISPLAY).unwrap();
    // Pick the first occupied unit box hit near the camera's center ray.
    // A fixed sub-voxel offset avoids grazing integer lattice corners. This is
    // a geometry selection, not rasterization; both images come only from Viewport.
    let mut candidates: Vec<_> = snapshot
        .composite()
        .into_keys()
        .filter_map(|p| {
            let mut enter = 0.0f64;
            let mut exit = f64::INFINITY;
            for (a, &coordinate) in p.iter().enumerate() {
                let direction = camera.target[a] - camera.eye[a];
                let origin = camera.eye[a] + [0.137, 0.271, 0.419][a];
                let t0 = (f64::from(coordinate) - origin) / direction;
                let t1 = (f64::from(coordinate + 1) - origin) / direction;
                enter = enter.max(t0.min(t1));
                exit = exit.min(t0.max(t1));
            }
            (exit - enter > 1e-6).then_some((enter, p))
        })
        .collect();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let at = candidates.first().expect("center ray intersects autumn").1;
    let (old_slot, layer) = snapshot.sample(at).unwrap();
    let old = &snapshot.palette.get(old_slot).unwrap().rgba;
    let slot = snapshot
        .palette
        .slots
        .iter()
        .max_by_key(|s| {
            s.color.rgba[..3]
                .iter()
                .zip(old)
                .map(|(a, b)| u32::from(a.abs_diff(*b)))
                .sum::<u32>()
        })
        .unwrap()
        .index;
    assert_ne!(slot, old_slot);
    let mut doc = Document::new(Uuid::from_u128(1), Uuid::from_u128(2));
    doc.content = snapshot.clone();
    let mut engine = Engine::new(doc).unwrap();
    let receipt = engine
        .execute(Transaction {
            expected_revision: engine.revision(),
            id: Uuid::new_v4(),
            actor: Actor::new(Origin::Human),
            label: "single voxel visual proof".into(),
            command: Command::SetVoxels {
                layer,
                voxels: vec![Voxel { at, slot }],
                mode: Mode::Paint,
                mirror: Mirror::default(),
            },
        })
        .unwrap();
    assert_eq!(receipt.changes.recolored, 1);
    assert_eq!(receipt.changes.added, 0);
    assert_eq!(receipt.changes.removed, 0);
    let mut gpu = block_on(Headless::new(HeadlessOptions::default())).unwrap();
    let golden = block_on(gpu.render(
        Some(snapshot.clone()),
        snapshot.bounds,
        camera,
        visual::DISPLAY,
        style(snapshot.bounds),
        true,
    ))
    .unwrap();
    let actual = block_on(gpu.render(
        Some(engine.document().content.clone()),
        snapshot.bounds,
        camera,
        visual::DISPLAY,
        style(snapshot.bounds),
        true,
    ))
    .unwrap();
    let (golden, actual) = (golden.rgba.unwrap(), actual.rgba.unwrap());
    let diff = visual::diff(&golden, &actual, 2).unwrap();
    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/visual/recolor-proof");
    let size = [1280, 800];
    let heat = visual::heatmap(&golden, &actual).unwrap();
    visual::write_png(&out.join("golden.png"), size, &golden).unwrap();
    visual::write_png(&out.join("actual.png"), size, &actual).unwrap();
    visual::write_png(&out.join("diff.png"), size, &heat).unwrap();
    visual::write_png(
        &out.join("side-by-side.png"),
        [3840, 800],
        &visual::side_by_side(size, &golden, &actual, &heat).unwrap(),
    )
    .unwrap();
    voxelite_bench::report::write_json(&out.join("metrics.json"), &diff).unwrap();
    println!("Exactly one voxel recolored at {at:?}, slot {old_slot} -> {slot}: {diff:?}");
    assert!(
        diff.fails(0.01, 24),
        "a single visible voxel recolor must FAIL default visual tolerance"
    );
}
