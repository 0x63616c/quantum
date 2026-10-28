use super::*;
use quantum_render::PresetView;
fn editor() -> Editor {
    Editor::blank().unwrap()
}
#[test]
fn tools_and_mirror_are_atomic_and_reversible() {
    let mut e = editor();
    let original = e.engine.document().clone();
    for axis in 0..3 {
        e.set_mirror_axis(axis, true).unwrap();
    }
    e.color = Swatch::rgb(2, 5, 8);
    assert!(e.apply_at([1, 2, 3]).unwrap());
    assert_eq!(e.snapshot().layers[0].voxels.len(), 8);
    assert_eq!(e.engine.history().count(), 1);
    e.undo().unwrap();
    assert_eq!(e.engine.document(), &original);
    assert!(!e.engine.is_dirty());
    e.redo().unwrap();
    assert_eq!(e.snapshot().layers[0].voxels.len(), 8);
}
#[test]
fn box_preview_cancels_without_mutation_and_commits_once() {
    let mut e = editor();
    e.tool = Tool::Box;
    assert!(!e.apply_at([1, 1, 1]).unwrap());
    assert!(!e.engine.is_dirty());
    e.box_start = None;
    e.apply_at([3, 3, 3]).unwrap();
    e.apply_at([2, 2, 2]).unwrap();
    assert_eq!(e.snapshot().layers[0].voxels.len(), 8);
    assert_eq!(e.engine.history().count(), 1);
}
#[test]
fn paint_fill_picker_erase_and_locks() {
    let mut e = editor();
    e.apply_at([1, 1, 1]).unwrap();
    e.apply_at([1, 1, 2]).unwrap();
    e.tool = Tool::Paint;
    e.color = Swatch::rgb(100, 50, 30);
    e.apply_at([1, 1, 1]).unwrap();
    e.tool = Tool::Picker;
    e.apply_at([1, 1, 2]).unwrap();
    assert_ne!(e.color.rgba, [100, 50, 30, 255]);
    e.tool = Tool::Fill;
    e.apply_at([1, 1, 1]).unwrap();
    assert_eq!(
        e.snapshot().layers[0].voxels.get([1, 1, 1]),
        e.snapshot().layers[0].voxels.get([1, 1, 2])
    );
    e.update_layer(e.layer, None, Some(true)).unwrap();
    let before = e.engine.document().clone();
    e.tool = Tool::Erase;
    assert!(e.apply_at([1, 1, 1]).is_err());
    assert_eq!(e.engine.document(), &before);
    e.update_layer(e.layer, None, Some(false)).unwrap();
    e.apply_at([1, 1, 1]).unwrap();
    assert_eq!(e.snapshot().layers[0].voxels.len(), 1);
}
#[test]
fn palette_choice_and_noop_do_not_recolor_or_dirty_document() {
    let mut e = editor();
    e.color = quantum_plugin_palette::Preset::Pico8
        .palette()
        .get(4)
        .unwrap()
        .clone();
    assert!(!e.engine.is_dirty());
    e.tool = Tool::Paint;
    assert!(!e.apply_at([2, 2, 2]).unwrap());
    assert_eq!(e.snapshot().palette.slots.len(), 1);
    assert!(!e.engine.is_dirty());
}
#[test]
fn fill_empty_cell_does_not_flood_space_or_import_a_color() {
    let mut e = editor();
    let original = e.engine.document().clone();
    e.tool = Tool::Fill;
    e.color = Swatch::rgb(1, 2, 3);
    assert!(!e.apply_at([2, 2, 2]).unwrap());
    assert_eq!(e.engine.document(), &original);
    assert_eq!(e.engine.revision(), 0);
    assert!(!e.engine.is_dirty());
}
#[test]
fn keyboard_picker_samples_visible_composite_with_locked_or_hidden_active_layer() {
    let mut e = editor();
    e.color = Swatch::rgb(1, 2, 3);
    e.apply_at([2, 2, 2]).unwrap();
    e.add_layer().unwrap();
    e.update_layer(e.layer, Some(false), Some(true)).unwrap();
    let revision = e.engine.revision();
    e.tool = Tool::Picker;
    e.color = Swatch::rgb(4, 5, 6);
    assert!(!e.apply_at([2, 2, 2]).unwrap());
    assert_eq!(e.color.rgba, [1, 2, 3, 255]);
    assert_eq!(e.engine.revision(), revision);
}
#[test]
fn save_rejects_external_changes_and_deletion_without_losing_edits() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("project.voxl");
    let mut e = editor();
    e.save(&path).unwrap();
    let mut external = Editor::open(&path).unwrap();
    e.apply_at([1, 2, 3]).unwrap();
    external.apply_at([4, 5, 6]).unwrap();
    external.save(&path).unwrap();
    let outside_bytes = std::fs::read(&path).unwrap();
    assert!(matches!(
        e.save(&path),
        Err(EditorError::Session(
            quantum_format::session::SessionError::ExternalChange
        ))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), outside_bytes);
    assert!(e.engine.is_dirty());
    std::fs::remove_file(&path).unwrap();
    assert!(matches!(
        e.save(&path),
        Err(EditorError::Session(
            quantum_format::session::SessionError::ExternalChange
        ))
    ));
    assert!(!path.exists());
    e.save(&dir.path().join("recovered.voxl")).unwrap();
    assert!(!e.engine.is_dirty());
}
#[test]
fn background_save_keeps_subsequent_edits_dirty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("project.voxl");
    let mut e = editor();
    e.apply_at([1, 2, 3]).unwrap();
    let save = e.prepare_save(&path);
    e.apply_at([4, 5, 6]).unwrap();
    e.finish_save(save.write().unwrap());
    assert!(e.engine.is_dirty());
    let opened = Editor::open(&path).unwrap();
    assert_eq!(opened.snapshot().layers[0].voxels.len(), 1);
    e.save(&path).unwrap();
    assert!(!e.engine.is_dirty());
    assert_eq!(
        Editor::open(&path).unwrap().snapshot().layers[0]
            .voxels
            .len(),
        2
    );
}
#[cfg(unix)]
#[test]
fn opening_symlink_saves_target_and_save_as_rejects_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("project.voxl");
    let link = dir.path().join("alias.voxl");
    let mut e = editor();
    e.save(&path).unwrap();
    std::os::unix::fs::symlink(&path, &link).unwrap();
    let mut opened = Editor::open(&link).unwrap();
    opened.apply_at([1, 2, 3]).unwrap();
    opened.save(&opened.path.clone().unwrap()).unwrap();
    assert!(link.is_symlink());
    assert_eq!(
        Editor::open(&path).unwrap().snapshot().layers[0]
            .voxels
            .len(),
        1
    );
    assert!(opened.save(&link).is_err());
}
#[test]
fn save_roundtrip_and_failed_save_preserve_dirty_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.voxl");
    let mut e = editor();
    e.apply_at([2, 3, 4]).unwrap();
    assert!(e.save(&dir.path().join("missing/test.voxl")).is_err());
    assert!(e.engine.is_dirty());
    e.save(&path).unwrap();
    assert!(!e.engine.is_dirty());
    assert_eq!(
        Editor::open(&path).unwrap().engine.document(),
        e.engine.document()
    );
    let bytes = std::fs::read(&path).unwrap();
    e.save(&path).unwrap();
    assert_eq!(bytes, std::fs::read(path).unwrap());
}
#[test]
fn layer_selection_reconciles_after_undo() {
    let mut e = editor();
    let first = e.layer;
    e.add_layer().unwrap();
    assert_ne!(e.layer, first);
    e.undo().unwrap();
    assert_eq!(e.layer, first);
    e.apply_at([1, 1, 1]).unwrap();
}
#[test]
fn camera_pose_roundtrips_and_interpolation_preserves_destination() {
    let e = editor();
    let a = OrbitCamera::fit_view(e.snapshot(), 1.3, PresetView::Left);
    let b = OrbitCamera::fit_view(e.snapshot(), 0.8, PresetView::Top);
    assert_eq!(OrbitCamera::from_pose(a.pose()), a);
    assert_eq!(OrbitCamera::from_pose(b.pose()), b);
    assert_eq!(OrbitCamera::from_pose(a.pose().lerp(b.pose(), 1.0)), b);
    assert_eq!(b.pitch, std::f64::consts::FRAC_PI_2);
}
#[test]
fn edits_undo_and_redo_leave_the_camera_unchanged() {
    let mut e = editor();
    e.camera.orbit(23.0, -7.0);
    e.camera.pan(20.0, -10.0, 600.0);
    e.camera.zoom(30.0);
    let camera = e.camera;
    e.apply_at([2, 3, 4]).unwrap();
    assert_eq!(e.camera, camera);
    e.undo().unwrap();
    assert_eq!(e.camera, camera);
    e.redo().unwrap();
    assert_eq!(e.camera, camera);
}
#[test]
fn rays_hit_nearest_visible_layer_and_placement_face() {
    let mut e = editor();
    e.apply_at([2, 2, 2]).unwrap();
    e.apply_at([2, 2, 4]).unwrap();
    let ray = Ray {
        origin: DVec3::new(2.5, 2.5, 40.0),
        direction: DVec3::NEG_Z,
    };
    let hit = raycast(e.snapshot(), ray).unwrap();
    assert_eq!(hit.at, [2, 2, 4]);
    assert_eq!(hit.normal, [0, 0, 1]);
    assert_eq!(e.target(ray), Some([2, 2, 5]));
    e.add_layer().unwrap();
    e.apply_at([2, 2, 4]).unwrap();
    assert_eq!(raycast(e.snapshot(), ray).unwrap().layer, e.layer);
    e.update_layer(e.layer, Some(false), None).unwrap();
    assert_ne!(raycast(e.snapshot(), ray).unwrap().layer, e.layer);
}
#[test]
fn negative_coordinates_parallel_misses_and_invalid_rays() {
    let mut e = editor();
    e.execute(
        "Resize",
        Command::EditDocument {
            name: None,
            bounds: Some(Region::new([-8; 3], [8; 3]).unwrap()),
        },
    )
    .unwrap();
    e.apply_at([-3, -2, -1]).unwrap();
    let ray = Ray {
        origin: DVec3::new(-2.5, -1.5, 20.0),
        direction: DVec3::NEG_Z,
    };
    assert_eq!(raycast(e.snapshot(), ray).unwrap().at, [-3, -2, -1]);
    assert!(
        raycast(
            e.snapshot(),
            Ray {
                origin: DVec3::splat(50.0),
                ..ray
            }
        )
        .is_none()
    );
    assert!(
        raycast(
            e.snapshot(),
            Ray {
                direction: DVec3::ZERO,
                ..ray
            }
        )
        .is_none()
    );
}
#[test]
fn camera_projection_and_ray_remain_inverse_after_navigation() {
    let e = editor();
    let mut c = e.camera;
    c.orbit(120.0, -30.0);
    c.pan(40.0, -20.0, 700.0);
    c.zoom(75.0);
    let p = DVec3::new(3.5, 4.5, 5.5);
    let uv = c.project(p, 1.4);
    let ray = c.ray(uv, 1.4).unwrap();
    assert!((p - ray.origin).cross(ray.direction).length() < 1e-10);
    for _ in 0..100 {
        c.orbit(20.0, 100.0);
        c.zoom(500.0);
    }
    assert!(c.camera().validate().is_ok());
    assert!(c.span >= 2.0);
    assert!(c.ray([f64::NAN, 0.5], 1.4).is_none());
}
#[test]
fn named_views_fit_bounds_and_top_points_exactly_down() {
    let mut e = editor();
    e.apply_at([0, 0, 0]).unwrap();
    e.apply_at([31, 10, 20]).unwrap();
    for view in [
        PresetView::Iso,
        PresetView::Front,
        PresetView::Right,
        PresetView::Top,
    ] {
        let camera = OrbitCamera::fit_view(e.snapshot(), 0.8, view);
        assert!(camera.camera().validate().is_ok());
        for bits in 0..8 {
            let corner = DVec3::new(
                if bits & 1 == 0 { 0.0 } else { 32.0 },
                if bits & 2 == 0 { 0.0 } else { 11.0 },
                if bits & 4 == 0 { 0.0 } else { 21.0 },
            );
            assert!(
                camera
                    .project(corner, 0.8)
                    .into_iter()
                    .all(|v| (0.0..=1.0).contains(&v))
            );
        }
        if view == PresetView::Top {
            let ray = camera.ray([0.5, 0.5], 0.8).unwrap();
            assert!((ray.direction - DVec3::NEG_Y).length() < 1e-12);
        }
    }
}
#[test]
fn asymmetric_scene_is_centered_and_clear_of_viewport_controls() {
    let e = Editor::example().unwrap();
    for aspect in [616.0 / 512.0, 1032.0 / 772.0, 0.8, 2.0] {
        for view in [
            PresetView::Iso,
            PresetView::Front,
            PresetView::Right,
            PresetView::Top,
        ] {
            let camera = OrbitCamera::fit_view(e.snapshot(), aspect, view);
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for layer in e.snapshot().layers.iter().filter(|layer| layer.visible) {
                for (position, _) in layer.voxels.iter() {
                    for bits in 0..8 {
                        let corner = DVec3::from(std::array::from_fn(|axis| {
                            f64::from(position[axis] + ((bits >> axis) & 1))
                        }));
                        for (axis, value) in camera.project(corner, aspect).into_iter().enumerate()
                        {
                            lo[axis] = lo[axis].min(value);
                            hi[axis] = hi[axis].max(value);
                        }
                    }
                }
            }
            for axis in 0..2 {
                assert!((lo[axis] + hi[axis] - 1.0).abs() < 1e-10);
                assert!(lo[axis] > 0.05 && hi[axis] < 0.95);
            }
            // The minimum window reserves the bottom 58px for navigation.
            if view == PresetView::Iso {
                assert!(hi[1] < 1.0 - 58.0 / 512.0);
            }
        }
    }
}
#[test]
fn ground_plane_allows_empty_document_editing() {
    let e = editor();
    let ray = Ray {
        origin: DVec3::new(4.5, 20.0, 5.5),
        direction: DVec3::NEG_Y,
    };
    assert_eq!(e.target(ray), Some([4, 0, 5]));
}
#[test]
fn example_loads_and_renders_without_display() {
    let e = Editor::example().unwrap();
    assert!(
        e.snapshot()
            .layers
            .iter()
            .map(|l| l.voxels.len())
            .sum::<usize>()
            > 1000
    );
    let image = quantum_render::render(
        e.snapshot(),
        &quantum_render::RenderOptions {
            width: 64,
            height: 64,
            supersampling: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(image.rgba.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
}
#[test]
fn foreign_layer_hit_is_not_silently_edited() {
    let mut e = editor();
    e.apply_at([2, 2, 2]).unwrap();
    e.add_layer().unwrap();
    e.tool = Tool::Erase;
    let before = e.engine.document().clone();
    let ray = Ray {
        origin: DVec3::new(2.5, 2.5, 40.0),
        direction: DVec3::NEG_Z,
    };
    assert!(e.click(ray, false).is_err());
    assert_eq!(e.engine.document(), &before);
    assert!(!e.click(ray, true).unwrap());
}
#[test]
fn save_preserves_forward_metadata_and_portable_view() {
    let base = editor();
    let project = Project::new(base.engine.document().clone()).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&project.to_bytes().unwrap()).unwrap();
    json["future_extension"] = serde_json::json!({"note": "retain me"});
    json["view"] = serde_json::json!({"camera": "iso"});
    let mut e = Editor::new(
        quantum_format::read(serde_json::to_vec(&json).unwrap().as_slice()).unwrap(),
        None,
    )
    .unwrap();
    e.apply_at([1, 2, 3]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("metadata.voxl");
    e.save(&path).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(saved["future_extension"], json["future_extension"]);
    assert_eq!(saved["view"], json["view"]);
}
#[test]
fn full_palette_failure_is_atomic_including_history() {
    let mut e = editor();
    let commands = (2..=256)
        .map(|slot| {
            Command::Palette(PaletteEdit::Add {
                slot: Some(slot),
                color: Swatch::rgb(0, (slot - 1) as u8, 0),
            })
        })
        .collect();
    e.execute("Fill palette", Command::Batch(commands)).unwrap();
    let before = e.engine.document().clone();
    let revision = e.engine.revision();
    e.color = Swatch::rgb(123, 45, 67);
    assert!(e.apply_at([1, 1, 1]).is_err());
    assert_eq!(e.engine.document(), &before);
    assert_eq!(e.engine.revision(), revision);
}
#[test]
fn hidden_layers_and_background_picker_leave_color_unchanged() {
    let mut e = editor();
    e.apply_at([2, 2, 2]).unwrap();
    e.update_layer(e.layer, Some(false), None).unwrap();
    let ray = Ray {
        origin: DVec3::new(2.5, 2.5, 40.0),
        direction: DVec3::NEG_Z,
    };
    assert!(raycast(e.snapshot(), ray).is_none());
    e.color = Swatch::rgb(23, 45, 67);
    let color = e.color.clone();
    assert!(!e.click(ray, true).unwrap());
    assert_eq!(e.color, color);
    assert!(e.apply_at([2, 2, 2]).is_err());
}
#[test]
fn projection_and_raycast_agree_across_orbits() {
    let mut e = editor();
    e.apply_at([15, 15, 15]).unwrap();
    let mut camera = OrbitCamera::fit(e.snapshot(), 1.4);
    for step in 0..80 {
        camera.yaw = f64::from(step) * 0.13;
        camera.pitch = -1.3 + f64::from(step % 20) * 0.13;
        let uv = camera.project(DVec3::splat(15.5), 1.4);
        let hit = raycast(e.snapshot(), camera.ray(uv, 1.4).unwrap()).unwrap();
        assert_eq!(hit.at, [15; 3]);
        assert_eq!(hit.normal.iter().map(|v| v.abs()).sum::<i32>(), 1);
    }
}
