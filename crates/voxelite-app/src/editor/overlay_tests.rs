//! Characterization captured before moving the original camera gizmo and grid.
use super::*;
use voxelite_plugin_api::{OverlayGesture, OverlayInput, OverlayNavigation, OverlayPrimitive};
use voxelite_plugin_gizmo_grid::{GIZMO_OVERLAY, PLUGIN_ID};

fn input(gesture: OverlayGesture, position: [f32; 2]) -> OverlayInput {
    OverlayInput {
        gesture,
        position,
        pointer: position,
        drag: None,
    }
}
fn hit(editor: &Editor, position: [f32; 2]) -> Option<u32> {
    editor
        .overlay_outcome(
            GIZMO_OVERLAY,
            editor.camera,
            input(OverlayGesture::Hover, position),
        )
        .unwrap()
        .and_then(|o| o.hovered)
}
fn widget(editor: &Editor) -> voxelite_plugin_api::OverlayWidget {
    editor
        .overlays(editor.camera, None)
        .into_iter()
        .flat_map(|f| f.primitives)
        .find_map(|p| match p {
            OverlayPrimitive::Widget(widget) => Some(widget),
            _ => None,
        })
        .unwrap()
}
fn snap(editor: &mut Editor, point: [f32; 2]) {
    let down = editor
        .interact_overlay(
            GIZMO_OVERLAY,
            editor.camera,
            input(OverlayGesture::Press { double: false }, point),
        )
        .unwrap()
        .unwrap();
    let mut release = input(OverlayGesture::Release, point);
    release.drag = down.drag;
    let outcome = editor
        .interact_overlay(GIZMO_OVERLAY, editor.camera, release)
        .unwrap()
        .unwrap();
    assert!(outcome.commands.is_empty());
    assert!(matches!(
        outcome.navigation,
        Some(OverlayNavigation::Orient { .. })
    ));
}

#[test]
fn every_end_snaps_without_moving_zooming_or_editing_and_poles_pick_frontmost() {
    for id in 0..6 {
        let mut editor = Editor::blank().unwrap();
        let before = editor.camera;
        let bytes = editor.project().to_bytes().unwrap();
        let circle = widget(&editor)
            .circles
            .into_iter()
            .find(|c| {
                c.fill
                    == if id >= 3 {
                        voxelite_plugin_api::OverlayColor::Axis(id % 3)
                    } else {
                        voxelite_plugin_api::OverlayColor::Background
                    }
                    && c.border == voxelite_plugin_api::OverlayColor::Axis(id % 3)
            })
            .unwrap();
        assert_eq!(hit(&editor, circle.center), Some(id as u32));
        snap(&mut editor, circle.center);
        let direction = [DVec3::X, DVec3::Y, DVec3::Z][id % 3] * if id >= 3 { 1. } else { -1. };
        let (right, up, back) = editor.camera.basis();
        assert!(back.abs_diff_eq(direction, 1e-12));
        assert!(right.dot(up).abs() < 1e-12);
        assert!((right.length() - 1.).abs() < 1e-12);
        assert!((up.length() - 1.).abs() < 1e-12);
        assert!(editor.camera.camera().up.iter().all(|v| v.is_finite()));
        assert_eq!(hit(&editor, [56.; 2]), Some(id as u32));
        assert_eq!(editor.camera.target, before.target);
        assert_eq!(editor.camera.span, before.span);
        assert_eq!(editor.project().to_bytes().unwrap(), bytes);
        assert_eq!(editor.engine.history().count(), 0);
    }
}

#[test]
fn picks_all_six_ends_and_rejects_empty_space_and_circle_corners() {
    let mut editor = Editor::blank().unwrap();
    for circle in widget(&editor).circles {
        assert!(hit(&editor, circle.center).is_some());
    }
    for point in [[0.; 2], [56.; 2], [f32::NAN, 0.]] {
        assert_eq!(hit(&editor, point), None);
    }
    let front = widget(&editor)
        .circles
        .into_iter()
        .find(|c| c.label.as_deref() == Some("Z"))
        .unwrap();
    snap(&mut editor, front.center);
    assert_eq!(hit(&editor, [56. + 38. + 9., 56. + 9.]), None);
}

#[test]
fn drag_threshold_uses_incremental_pixels_and_never_edits_the_document() {
    let mut editor = Editor::blank().unwrap();
    let bytes = editor.project().to_bytes().unwrap();
    let before = editor.camera;
    let mut outcome = editor
        .interact_overlay(
            GIZMO_OVERLAY,
            editor.camera,
            input(OverlayGesture::Press { double: false }, [10., 20.]),
        )
        .unwrap()
        .unwrap();
    for (point, navigation) in [
        ([12., 20.], None),
        ([13., 20.], Some(OverlayNavigation::Orbit([1., 0.]))),
        ([11., 21.], Some(OverlayNavigation::Orbit([-2., 1.]))),
    ] {
        let mut next = input(OverlayGesture::Move { primary_down: true }, point);
        next.drag = outcome.drag;
        outcome = editor
            .interact_overlay(GIZMO_OVERLAY, editor.camera, next)
            .unwrap()
            .unwrap();
        assert_eq!(outcome.navigation, navigation);
        assert!(outcome.commands.is_empty());
    }
    assert_eq!(
        editor.camera.yaw,
        ((before.yaw - 0.008).rem_euclid(std::f64::consts::TAU) + 0.016)
            .rem_euclid(std::f64::consts::TAU)
    );
    assert_eq!(editor.camera.pitch, before.pitch + 0.008);
    assert_eq!(editor.camera.target, before.target);
    assert_eq!(editor.camera.span, before.span);
    assert_eq!(editor.project().to_bytes().unwrap(), bytes);
    assert_eq!(editor.engine.history().count(), 0);
}

#[test]
fn disabled_and_absent_overlays_leave_editing_navigation_saving_and_undo_usable() {
    for absent in [false, true] {
        let mut editor = Editor::blank().unwrap();
        editor.set_grid(false).unwrap();
        if absent {
            editor.plugins = RefCell::new(Host::new(vec![Box::new(
                voxelite_plugin_tools::ToolsPlugin,
            )]));
        } else {
            editor.set_plugin_enabled(PLUGIN_ID, false).unwrap();
        }
        assert!(!editor.grid());
        assert!(editor.overlays(editor.camera, None).is_empty());
        assert!(
            editor
                .interact_overlay(
                    GIZMO_OVERLAY,
                    editor.camera,
                    input(OverlayGesture::Press { double: true }, [0.; 2])
                )
                .unwrap()
                .is_none()
        );
        let before = editor.project().to_bytes().unwrap();
        editor.camera.orbit(10., 10.);
        assert!(editor.apply_at([1; 3]).unwrap());
        let after = editor.project().to_bytes().unwrap();
        editor.undo().unwrap();
        assert_eq!(editor.project().to_bytes().unwrap(), before);
        editor.redo().unwrap();
        assert_eq!(editor.project().to_bytes().unwrap(), after);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("disabled.voxl");
        editor.save(&path).unwrap();
        assert_eq!(
            Editor::open(&path).unwrap().project().to_bytes().unwrap(),
            after
        );
        if !absent {
            editor.set_plugin_enabled(PLUGIN_ID, true).unwrap();
            assert!(!editor.grid());
            assert_eq!(widget(&editor).circles.len(), 6);
            editor.set_grid(true).unwrap();
            assert!(editor.grid());
        }
    }
}

#[test]
fn grid_preference_bridge_keeps_existing_bytes_defaults_and_document_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut preferences = crate::preferences::Preferences::load(&path).unwrap();
    let mut editor = Editor::blank().unwrap();
    let before = editor.project().to_bytes().unwrap();
    assert!(editor.grid() && preferences.grid);
    preferences.grid = !preferences.grid;
    preferences.save(&path).unwrap();
    editor.set_grid(preferences.grid).unwrap();
    assert!(!editor.grid());
    assert!(
        editor
            .overlays(editor.camera, None)
            .into_iter()
            .flat_map(|f| f.primitives)
            .all(|p| !matches!(p, OverlayPrimitive::Grid(_)))
    );
    assert_eq!(editor.project().to_bytes().unwrap(), before);
    let mut reopened = Editor::new(editor.project(), None).unwrap();
    reopened
        .set_grid(crate::preferences::Preferences::load(&path).unwrap().grid)
        .unwrap();
    assert!(!reopened.grid());
    assert_eq!(reopened.project().to_bytes().unwrap(), before);
    assert_eq!(reopened.engine.history().count(), 0);
}

#[test]
fn registered_overlay_edits_are_atomic_typed_commands_before_navigation_feedback() {
    use voxelite_plugin_api::{
        Overlay, OverlayCx, Plugin, Registrar, RenderPassContribution, RenderPassSlot,
    };
    struct EditingOverlay {
        commands: Vec<Command>,
    }
    impl Plugin for EditingOverlay {
        fn manifest(&self) -> voxelite_plugin_api::Manifest {
            voxelite_plugin_gizmo_grid::GizmoGridPlugin.manifest()
        }
        fn contribute(&self, registrar: &mut Registrar) {
            registrar.render_pass(RenderPassContribution {
                id: "dev.test.edit".into(),
                slot: RenderPassSlot::Overlay,
            });
        }
        fn overlay(&self, id: &str) -> Option<&dyn Overlay> {
            (id == "dev.test.edit").then_some(self)
        }
    }
    impl Overlay for EditingOverlay {
        fn draw(&self, _: &mut dyn OverlayCx, _: Option<u32>) {}
        fn interact(
            &self,
            _: &dyn OverlayCx,
            _: OverlayInput,
        ) -> std::result::Result<OverlayOutcome, ToolError> {
            Ok(OverlayOutcome {
                commands: self.commands.clone(),
                label: "Overlay edit".into(),
                navigation: Some(OverlayNavigation::Orbit([5., 2.])),
                ..Default::default()
            })
        }
    }
    for rejected in [false, true] {
        let mut editor = Editor::blank().unwrap();
        let before = editor.project().to_bytes().unwrap();
        let camera = editor.camera;
        let commands = vec![Command::SetVoxels {
            mode: Mode::Place,
            layer: editor.layer,
            voxels: vec![Voxel {
                at: [1; 3],
                slot: if rejected { u16::MAX } else { 1 },
            }],
            mirror: Mirror::default(),
        }];
        editor.plugins = RefCell::new(Host::new(vec![Box::new(EditingOverlay {
            commands: commands.clone(),
        })]));
        let outcome = editor.interact_overlay(
            "dev.test.edit",
            camera,
            input(OverlayGesture::Release, [0.; 2]),
        );
        assert_eq!(
            editor.last_command,
            Some(("Overlay edit".into(), Command::Batch(commands)))
        );
        if rejected {
            assert!(outcome.is_err());
            assert_eq!(editor.camera, camera);
            assert_eq!(editor.project().to_bytes().unwrap(), before);
            assert_eq!(editor.engine.history().count(), 0);
        } else {
            assert!(outcome.unwrap().is_some());
            assert_ne!(editor.camera, camera);
            let entry = editor.engine.history().next_back().unwrap();
            assert_eq!(entry.label, "Overlay edit");
            assert_eq!(entry.actor.origin, Origin::Human);
            assert_eq!(entry.actor.tool, "native-editor");
            assert_eq!(editor.engine.history().count(), 1);
            let after = editor.project().to_bytes().unwrap();
            editor.undo().unwrap();
            assert_eq!(editor.project().to_bytes().unwrap(), before);
            editor.redo().unwrap();
            assert_eq!(editor.project().to_bytes().unwrap(), after);
        }
    }
}
