use super::*;
use voxelite_plugin_host::Host;
use voxelite_render::{Lighting, RasterCache, RenderOptions, render};

fn host() -> Host {
    Host::new(vec![Box::new(ShadowsLightingPlugin)])
}

fn fixture() -> Snapshot {
    json::from_slice(include_bytes!(
        "../../../voxelite-export/tests/golden/scene.json"
    ))
    .unwrap()
}

#[test]
fn conforms_to_plugin_contract() {
    voxelite_plugin_host::conformance::check(|| Box::new(ShadowsLightingPlugin));
}

#[test]
fn enabled_and_ambient_golden_images_and_cached_reenable() {
    let snapshot = fixture();
    let mut host = host();
    let cache = RasterCache::default();
    let mut options = RenderOptions {
        width: 80,
        height: 64,
        passes: host.render_plan(),
        ..Default::default()
    };
    assert_eq!(options.passes.lighting(), Lighting::Exact);
    let enabled = render(&snapshot, &options).unwrap();
    assert_eq!(cache.render(&snapshot, &options).unwrap(), enabled);
    // Reuse the pre-plugin PNG unchanged, including its exact encoding bytes.
    assert_eq!(
        encoding::png_rgba(enabled.width, enabled.height, &enabled.rgba).unwrap(),
        include_bytes!("../../../voxelite-export/tests/golden/scene.png")
    );
    host.disable(PLUGIN_ID).unwrap();
    options.passes = host.render_plan();
    assert_eq!(options.passes.lighting(), Lighting::Ambient);
    let ambient = render(&snapshot, &options).unwrap();
    assert_eq!(cache.render(&snapshot, &options).unwrap(), ambient);
    assert_eq!(render(&snapshot, &options).unwrap(), ambient);
    let png = encoding::png_rgba(ambient.width, ambient.height, &ambient.rgba).unwrap();
    assert_eq!(png, include_bytes!("../tests/golden/ambient.png"));
    assert_ne!(ambient.rgba, enabled.rgba);
    assert_eq!(ambient.camera, enabled.camera);
    assert_eq!(ambient.quads, enabled.quads);
    assert!(
        ambient
            .rgba
            .chunks_exact(4)
            .zip(enabled.rgba.chunks_exact(4))
            .all(|(a, b)| a[3] == b[3])
    );

    // Legacy AO/shadow options have no effect when their owning pass is absent.
    options.ambient_occlusion = false;
    options.shadows = false;
    assert_eq!(render(&snapshot, &options).unwrap(), ambient);
    options.ambient_occlusion = true;
    options.shadows = true;
    host.enable(PLUGIN_ID).unwrap();
    options.passes = host.render_plan();
    assert_eq!(cache.render(&snapshot, &options).unwrap(), enabled);
    assert_eq!(
        host.timings()
            .get(PLUGIN_ID, &format!("render_pass:{SURFACE_PASS}"))
            .unwrap()
            .count,
        2
    );
}

#[test]
fn edits_and_visibility_while_disabled_are_seen_on_reenable() {
    let mut snapshot = fixture();
    let cache = RasterCache::default();
    let mut host = host();
    let mut options = RenderOptions {
        width: 80,
        height: 64,
        passes: host.render_plan(),
        ..Default::default()
    };
    cache.render(&snapshot, &options).unwrap();
    host.disable(PLUGIN_ID).unwrap();
    options.passes = host.render_plan();
    snapshot.layers[0].voxels = Default::default();
    snapshot.layers[1].visible = true;
    snapshot.layers[1].voxels =
        json::from_value(json::json!([[0, 0, 0, 1], [0, 1, 0, 2], [1, 0, 0, 3]])).unwrap();
    assert_eq!(
        cache.render(&snapshot, &options).unwrap(),
        render(&snapshot, &options).unwrap()
    );
    host.enable(PLUGIN_ID).unwrap();
    options.passes = host.render_plan();
    assert_eq!(
        cache.render(&snapshot, &options).unwrap(),
        render(&snapshot, &options).unwrap()
    );
    // Remove all casters while disabled, then re-enable at the same camera.
    host.disable(PLUGIN_ID).unwrap();
    options.passes = host.render_plan();
    snapshot.layers[1].visible = false;
    cache.render(&snapshot, &options).unwrap();
    host.enable(PLUGIN_ID).unwrap();
    options.passes = host.render_plan();
    assert_eq!(
        cache.render(&snapshot, &options).unwrap(),
        render(&snapshot, &options).unwrap()
    );
}
