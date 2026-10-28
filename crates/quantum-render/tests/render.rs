//! Camera validation, raster visibility, alpha and reproducibility contracts.
use quantum_core::*;
use quantum_render::*;
#[expect(
    clippy::unwrap_used,
    reason = "test fixture or assertion helper reports invalid setup by panicking"
)]
fn scene() -> Snapshot {
    let mut e = Engine::new(Document::new(Uuid::from_u128(1), Uuid::from_u128(2))).unwrap();
    e.execute(Transaction {
        expected_revision: 0,
        id: Uuid::from_u128(3),
        actor: Actor::new(Origin::Human),
        label: "Cube".into(),
        command: Command::Primitive {
            layer: Uuid::from_u128(2),
            shape: Shape::Box {
                region: Region::new([0; 3], [3; 3]).unwrap(),
                hollow: false,
            },
            mode: Mode::Place,
            slot: Some(1),
            mirror: Mirror::default(),
        },
    })
    .unwrap();
    e.document().content.clone()
}
fn options() -> RenderOptions {
    RenderOptions {
        width: 64,
        height: 64,
        ..RenderOptions::default()
    }
}
#[test]
fn every_view_is_fitted_visible_and_deterministic() {
    let s = scene();
    for direction in [
        PresetView::Iso,
        PresetView::Front,
        PresetView::Back,
        PresetView::Left,
        PresetView::Right,
        PresetView::Top,
    ] {
        for perspective in [false, true] {
            let o = RenderOptions {
                view: View::Preset {
                    direction,
                    perspective,
                },
                ..options()
            };
            let a = render(&s, &o).unwrap();
            assert_eq!(a, render(&s, &o).unwrap());
            let occupied = a
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] > 0)
                .count();
            assert!(occupied > 500);
            assert!(occupied < 64 * 64);
            for x in 0..64 {
                assert_eq!(a.rgba[x * 4 + 3], 0);
                assert_eq!(a.rgba[((63 * 64) + x) * 4 + 3], 0);
            }
        }
    }
}
#[test]
fn supersampling_produces_coverage_without_dark_fringe() {
    let s = scene();
    let image = render(&s, &options()).unwrap();
    let pixels = image.rgba.as_chunks::<4>().0;
    assert!(pixels.iter().any(|p| p[3] > 0 && p[3] < 255));
    for p in pixels.iter().filter(|p| p[3] > 0 && p[3] < 255) {
        assert!(p[0] > 50);
    }
}
#[test]
fn empty_scene_background_and_hidden_layers() {
    let mut s = scene();
    s.layers[0].visible = false;
    let o = RenderOptions {
        background: [33, 66, 99, 128],
        ..options()
    };
    let image = render(&s, &o).unwrap();
    assert!(
        image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [33, 66, 99, 128])
    );
    assert_eq!(image.quads, 0);
}
#[test]
fn malformed_camera_and_target_budgets_reject() {
    let mut c = Camera::fit(&scene(), PresetView::Iso, 1.0, false).unwrap();
    c.eye = c.target;
    assert!(c.validate().is_err());
    c.eye = [f64::NAN, 0.0, 0.0];
    assert!(c.validate().is_err());
    c.eye = [0.0, 0.0, 3.0];
    c.target = [0.0; 3];
    c.up = [0.0, 0.0, 1.0];
    assert!(c.validate().is_err());
    for (w, h, aa) in [(0, 64, 2), (1025, 64, 2), (1024, 1024, 4), (64, 64, 0)] {
        assert!(
            render(
                &scene(),
                &RenderOptions {
                    width: w,
                    height: h,
                    supersampling: aa,
                    ..options()
                }
            )
            .is_err()
        );
    }
}
#[test]
fn explicit_perspective_clips_geometry_at_eye_plane() {
    let camera = Camera {
        eye: [1.5, 1.5, 1.5],
        target: [1.5, 1.5, -4.0],
        up: [0.0, 1.0, 0.0],
        projection: Projection::Perspective {
            fov_y_degrees: 70.0,
        },
    };
    let image = render(
        &scene(),
        &RenderOptions {
            view: View::Camera(camera),
            ..options()
        },
    )
    .unwrap();
    assert_eq!(image.rgba.len(), 64 * 64 * 4);
}
#[test]
fn outline_changes_colors_but_preserves_coverage() {
    let s = scene();
    let a = render(&s, &options()).unwrap();
    let b = render(
        &s,
        &RenderOptions {
            outline: true,
            ..options()
        },
    )
    .unwrap();
    assert_ne!(a.rgba, b.rgba);
    assert!(
        a.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .zip(b.rgba.as_chunks::<4>().0)
            .all(|(a, b)| a[3] == b[3])
    );
}
#[test]
fn nearest_surface_wins_and_hidden_back_cells_do_not_bleed() {
    let mut s = scene();
    s.palette = Palette::from_colors(vec![Swatch::rgb(255, 0, 0), Swatch::rgb(0, 0, 255)]).unwrap();
    let mut e = Engine::new(Document {
        id: Uuid::from_u128(1),
        content: s,
        versions: Default::default(),
        lineage: None,
        extensions: Default::default(),
    })
    .unwrap();
    e.execute(Transaction {
        expected_revision: 0,
        id: Uuid::from_u128(5),
        actor: Actor::new(Origin::Human),
        label: "Front blue".into(),
        command: Command::Primitive {
            layer: Uuid::from_u128(2),
            shape: Shape::Box {
                region: Region::new([0, 0, 3], [3, 3, 4]).unwrap(),
                hollow: false,
            },
            mode: Mode::Place,
            slot: Some(2),
            mirror: Mirror::default(),
        },
    })
    .unwrap();
    let image = render(
        &e.document().content,
        &RenderOptions {
            view: View::Preset {
                direction: PresetView::Front,
                perspective: false,
            },
            ..options()
        },
    )
    .unwrap();
    let center = &image.rgba[(32 * 64 + 32) * 4..][..4];
    assert_eq!(center[0], 0);
    assert!(center[2] > 150);
    assert_eq!(center[3], 255);
}

#[test]
fn cached_raster_matches_stateless_through_camera_and_visibility_changes() {
    let mut snapshot = scene();
    let cache = RasterCache::default();
    for direction in [PresetView::Iso, PresetView::Back, PresetView::Top] {
        let options = RenderOptions {
            view: View::Preset {
                direction,
                perspective: false,
            },
            ..options()
        };
        for visible in [true, false, true] {
            snapshot.layers[0].visible = visible;
            assert_eq!(
                cache.render(&snapshot, &options).unwrap(),
                render(&snapshot, &options).unwrap()
            );
        }
    }
}

#[test]
fn ao_strength_preserves_default_pixels_and_only_darkens_contacts() {
    // Three touching voxels form a concave step; a convex cube alone has no AO.
    let snapshot: Snapshot = serde_json::from_slice(include_bytes!(
        "../../plugins/quantum-plugin-exporters/tests/golden/scene.json"
    ))
    .unwrap();
    let cache = RasterCache::default();
    for shadows in [false, true] {
        let mut options = RenderOptions {
            shadows,
            ..options()
        };
        let original = render(&snapshot, &options).unwrap();
        options.ambient_occlusion_strength = AmbientOcclusionStrength::new(1.0).unwrap();
        assert_eq!(original, render(&snapshot, &options).unwrap());
        options.ambient_occlusion = false;
        let disabled = render(&snapshot, &options).unwrap();
        options.ambient_occlusion = true;
        options.ambient_occlusion_strength = AmbientOcclusionStrength::OFF;
        assert_eq!(disabled, render(&snapshot, &options).unwrap());
        assert_ne!(disabled.rgba, original.rgba);

        let mut previous = disabled;
        for strength in [0.5, 1.0, 2.0] {
            options.ambient_occlusion_strength = AmbientOcclusionStrength::new(strength).unwrap();
            let next = render(&snapshot, &options).unwrap();
            assert_eq!(next, cache.render(&snapshot, &options).unwrap());
            assert_eq!(next.camera, previous.camera);
            assert_eq!(next.quads, previous.quads);
            assert_ne!(next.rgba, previous.rgba);
            for (a, b) in previous
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .zip(next.rgba.as_chunks::<4>().0)
            {
                assert_eq!(a[3], b[3], "AO must preserve coverage");
                assert!((0..3).all(|c| b[c] <= a[c]), "increasing AO only darkens");
            }
            previous = next;
        }
    }
}

#[test]
fn ao_leaves_unoccluded_surfaces_unchanged() {
    let snapshot = scene();
    let mut options = options();
    let original = render(&snapshot, &options).unwrap();
    for strength in [0.0, 0.5, 2.0] {
        options.ambient_occlusion_strength = AmbientOcclusionStrength::new(strength).unwrap();
        assert_eq!(original, render(&snapshot, &options).unwrap());
    }
}

#[cfg(feature = "profiling")]
#[test]
fn profiling_enters_render_and_mesh_zones_without_changing_pixels() {
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

    struct Zones(Arc<Mutex<Vec<&'static str>>>);
    impl<S: tracing::Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Zones {
        fn on_enter(&self, id: &tracing::span::Id, ctx: Context<'_, S>) {
            self.0.lock().unwrap().push(ctx.span(id).unwrap().name());
        }
    }

    let snapshot = scene();
    let options = options();
    let expected = render(&snapshot, &options).unwrap();
    let zones = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(Zones(Arc::clone(&zones)));
    let actual = tracing::subscriber::with_default(subscriber, || {
        RasterCache::default().render(&snapshot, &options).unwrap()
    });
    assert_eq!(actual.rgba, expected.rgba);
    let names = zones.lock().unwrap();
    for name in ["render.cpu.cached", "render.cpu", "mesh.greedy"] {
        assert!(names.contains(&name), "missing entered zone: {name}");
    }
}
