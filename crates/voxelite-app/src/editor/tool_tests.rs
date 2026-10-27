//! Characterization of the original editor gestures, shared by the plugin migration.
use super::*;

fn front_ray() -> Ray {
    Ray {
        origin: DVec3::new(2.5, 2.5, 40.0),
        direction: DVec3::NEG_Z,
    }
}

#[test]
fn pointer_gestures_preserve_commands_preview_bytes_and_undo() {
    for tool in Tool::ALL {
        let mut e = Editor::blank().unwrap();
        e.apply_at([2, 2, 2]).unwrap();
        e.apply_at([3, 2, 2]).unwrap();
        let before = e.project().to_bytes().unwrap();
        let history = e.engine.history().count();
        let sampled = e.color.clone();
        e.color = Swatch::rgb(17, 34, 51);
        e.tool = tool;
        e.last_command = None;
        let at = if matches!(tool, Tool::Place | Tool::Box) {
            [2, 2, 3]
        } else {
            [2, 2, 2]
        };
        assert_eq!(e.target(front_ray()), Some(at), "{tool:?}");
        assert_eq!(
            e.tool_preview(e.target(front_ray())),
            Some(Region::new(at, at.map(|v| v + 1)).unwrap()),
            "{tool:?}"
        );
        let changed = e.click(front_ray(), false).unwrap();
        if tool == Tool::Picker {
            assert!(!changed);
            assert_eq!(e.color, sampled);
            assert_eq!(e.last_command, None);
            assert_eq!(e.project().to_bytes().unwrap(), before);
            assert_eq!(e.engine.history().count(), history);
            continue;
        }
        let mut commands = Vec::new();
        let slot = if tool == Tool::Erase { 0 } else { 2 };
        if tool != Tool::Erase {
            commands.push(Command::Palette(PaletteEdit::Add {
                slot: Some(slot),
                color: e.color.clone(),
            }));
        }
        let mirror = Mirror::default();
        commands.push(match tool {
            Tool::Box => {
                assert!(!changed);
                assert_eq!(e.box_start, Some(at));
                assert_eq!(e.last_command, None);
                assert_eq!(e.project().to_bytes().unwrap(), before);
                assert_eq!(e.engine.history().count(), history);
                let ray = Ray {
                    origin: DVec3::new(4.5, 40.0, 4.5),
                    direction: DVec3::NEG_Y,
                };
                assert_eq!(e.target(ray), Some([4, 0, 4]));
                let region = Region::new([2, 0, 3], [5, 3, 5]).unwrap();
                assert_eq!(e.tool_preview(e.target(ray)), Some(region));
                assert_eq!(
                    e.tool_preview(None),
                    Some(Region::new(at, [3, 3, 4]).unwrap())
                );
                assert!(e.click(ray, false).unwrap());
                Command::Primitive {
                    layer: e.layer,
                    shape: Shape::Box {
                        region,
                        hollow: false,
                    },
                    mode: Mode::Place,
                    slot: Some(slot),
                    mirror,
                }
            }
            Tool::Fill => Command::FloodFill {
                layer: e.layer,
                seed: at,
                slot,
                region: None,
                cap: 1_048_576,
                mirror,
            },
            _ => Command::SetVoxels {
                layer: e.layer,
                voxels: vec![Voxel { at, slot }],
                mode: match tool {
                    Tool::Erase => Mode::Erase,
                    Tool::Paint => Mode::Paint,
                    _ => Mode::Place,
                },
                mirror,
            },
        });
        assert_eq!(
            e.last_command,
            Some((tool.name().into(), Command::Batch(commands)))
        );
        assert_eq!(e.box_start, None);
        assert_eq!(e.engine.history().count(), history + 1);
        let entry = e.engine.history().next_back().unwrap();
        assert_eq!(entry.label, tool.name());
        assert_eq!(entry.actor.origin, Origin::Human);
        assert_eq!(entry.actor.tool, "native-editor");
        let after = e.project().to_bytes().unwrap();
        assert_ne!(after, before);
        e.undo().unwrap();
        assert_eq!(e.project().to_bytes().unwrap(), before, "{tool:?}");
        e.redo().unwrap();
        assert_eq!(e.project().to_bytes().unwrap(), after, "{tool:?}");
    }
}

#[test]
fn alt_sampling_preserves_pending_box_and_never_produces_commands() {
    let mut e = Editor::blank().unwrap();
    e.apply_at([2, 2, 2]).unwrap();
    let sampled = e.color.clone();
    let before = e.project().to_bytes().unwrap();
    let revision = e.engine.revision();
    for tool in Tool::ALL {
        e.tool = tool;
        e.box_start = Some([1, 1, 1]);
        e.color = Swatch::rgb(17, 34, 51);
        e.last_command = None;
        assert!(!e.click(front_ray(), true).unwrap());
        assert_eq!(e.color, sampled);
        assert_eq!(e.box_start, Some([1, 1, 1]));
        assert_eq!(e.last_command, None);
        assert_eq!(e.project().to_bytes().unwrap(), before);
        assert_eq!(e.engine.revision(), revision);
    }
}

#[test]
fn noops_errors_and_outside_faces_preserve_gesture_state() {
    let mut e = Editor::blank().unwrap();
    e.color = Swatch::rgb(17, 34, 51);
    let before = e.project().to_bytes().unwrap();
    for tool in [Tool::Erase, Tool::Paint, Tool::Fill, Tool::Picker] {
        e.tool = tool;
        assert_eq!(e.target(front_ray()), None);
        assert!(!e.click(front_ray(), false).unwrap());
        assert!(!e.apply_at([2, 2, 2]).unwrap());
        assert_eq!(e.last_command, None);
        assert_eq!(e.project().to_bytes().unwrap(), before);
    }
    for tool in Tool::ALL {
        e.tool = tool;
        assert_eq!(
            e.apply_at([32, 0, 0]).unwrap_err().to_string(),
            "Outside document bounds"
        );
    }
    e.tool = Tool::Place;
    e.apply_at([2, 2, 31]).unwrap();
    for tool in [Tool::Place, Tool::Box] {
        e.tool = tool;
        assert_eq!(e.target(front_ray()), None);
        assert!(!e.click(front_ray(), false).unwrap());
    }
}

#[test]
fn editing_checks_hit_layer_before_active_layer_and_preserves_box_on_failure() {
    let mut e = Editor::blank().unwrap();
    e.apply_at([2, 2, 2]).unwrap();
    e.add_layer().unwrap();
    e.update_layer(e.layer, None, Some(true)).unwrap();
    let before = e.project().to_bytes().unwrap();
    let revision = e.engine.revision();
    for tool in [Tool::Paint, Tool::Erase, Tool::Fill] {
        e.tool = tool;
        assert_eq!(
            e.click(front_ray(), false).unwrap_err().to_string(),
            "Select the hit voxel’s layer to edit it"
        );
        assert_eq!(
            e.apply_at([2, 2, 2]).unwrap_err().to_string(),
            "The active layer is locked or hidden"
        );
    }
    e.tool = Tool::Box;
    e.box_start = Some([1, 1, 1]);
    assert!(e.apply_at([2, 2, 2]).is_err());
    assert_eq!(e.box_start, Some([1, 1, 1]));
    assert_eq!(e.project().to_bytes().unwrap(), before);
    assert_eq!(e.engine.revision(), revision);
}

#[test]
fn every_edit_tool_retains_centered_mirrors_and_atomic_undo() {
    for tool in [Tool::Place, Tool::Erase, Tool::Paint, Tool::Fill, Tool::Box] {
        let mut e = Editor::blank().unwrap();
        e.mirror = [true; 3];
        e.apply_at([2, 2, 2]).unwrap();
        let before = e.project().to_bytes().unwrap();
        let history = e.engine.history().count();
        e.tool = tool;
        e.color = Swatch::rgb(17, 34, 51);
        if tool == Tool::Box {
            assert!(!e.apply_at([2, 2, 2]).unwrap());
        }
        assert!(e.apply_at([2, 2, 2]).unwrap());
        let (_, Command::Batch(commands)) = e.last_command.as_ref().unwrap() else {
            panic!("tools execute one batch");
        };
        let mirror = match commands.last().unwrap() {
            Command::SetVoxels { mirror, .. }
            | Command::FloodFill { mirror, .. }
            | Command::Primitive { mirror, .. } => *mirror,
            other => panic!("unexpected command: {other:?}"),
        };
        assert_eq!(mirror, Mirror::centered(e.snapshot().bounds, [true; 3]));
        assert_eq!(
            e.snapshot().layers[0].voxels.len(),
            if tool == Tool::Erase { 0 } else { 8 }
        );
        assert_eq!(e.engine.history().count(), history + 1);
        let after = e.project().to_bytes().unwrap();
        e.undo().unwrap();
        assert_eq!(e.project().to_bytes().unwrap(), before, "{tool:?}");
        e.redo().unwrap();
        assert_eq!(e.project().to_bytes().unwrap(), after, "{tool:?}");
    }
}

#[test]
fn rejected_box_batch_keeps_anchor_and_rolls_back_palette() {
    let project = Editor::blank().unwrap().project();
    let session = DocumentSession::new(
        project,
        None,
        Limits {
            work_units: 1,
            ..Limits::default()
        },
    )
    .unwrap();
    let mut e = Editor::from_session(session).unwrap();
    let before = e.project().to_bytes().unwrap();
    e.tool = Tool::Box;
    e.color = Swatch::rgb(17, 34, 51);
    assert!(!e.apply_at([1, 1, 1]).unwrap());
    assert!(e.apply_at([4, 4, 4]).is_err());
    assert_eq!(e.box_start, Some([1, 1, 1]));
    assert_eq!(e.project().to_bytes().unwrap(), before);
    assert_eq!(e.engine.revision(), 0);
    assert_eq!(e.engine.history().count(), 0);
}

#[test]
fn disabling_tools_makes_all_gestures_inert_while_core_editor_still_works() {
    let mut e = Editor::blank().unwrap();
    e.apply_at([2; 3]).unwrap();
    e.tool = Tool::Box;
    e.apply_at([1; 3]).unwrap();
    e.set_plugin_enabled("dev.voxelite.tools", false).unwrap();
    assert_eq!(e.box_start, None);
    assert_eq!(e.plugin_host().registry().tools().count(), 0);
    assert_eq!(
        e.plugin_host().state("dev.voxelite.core"),
        Some(&voxelite_plugin_host::PluginState::Enabled)
    );
    let before = e.project().to_bytes().unwrap();
    let revision = e.engine.revision();
    let history = e.engine.history().count();
    let calls = e
        .plugin_host()
        .timings()
        .get("dev.voxelite.tools", "tool")
        .unwrap()
        .count;
    e.color = Swatch::rgb(17, 34, 51);
    for tool in Tool::ALL {
        e.tool = tool;
        e.last_command = None;
        assert_eq!(e.target(front_ray()), None);
        assert_eq!(e.tool_preview(Some([2; 3])), None);
        assert!(!e.click(front_ray(), false).unwrap());
        assert!(!e.click(front_ray(), true).unwrap());
        assert!(!e.apply_at([2; 3]).unwrap());
        assert_eq!(e.last_command, None);
        assert_eq!(e.color, Swatch::rgb(17, 34, 51));
        assert_eq!(e.project().to_bytes().unwrap(), before);
        assert_eq!(e.engine.revision(), revision);
        assert_eq!(e.engine.history().count(), history);
    }
    assert_eq!(
        e.plugin_host()
            .timings()
            .get("dev.voxelite.tools", "tool")
            .unwrap()
            .count,
        calls
    );
    let camera = e.camera;
    e.camera.orbit(20.0, 10.0);
    e.camera.zoom(30.0);
    assert_ne!(e.camera, camera);
    assert!(e.camera.camera().validate().is_ok());
    e.undo().unwrap();
    assert_eq!(e.snapshot().layers[0].voxels.len(), 0);
    e.redo().unwrap();
    assert_eq!(e.project().to_bytes().unwrap(), before);
    e.add_layer().unwrap();
    e.undo().unwrap();
    assert_eq!(e.project().to_bytes().unwrap(), before);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("without-tools.voxl");
    e.save(&path).unwrap();
    assert_eq!(
        Editor::open(&path).unwrap().project().to_bytes().unwrap(),
        before
    );
    e.set_plugin_enabled("dev.voxelite.tools", true).unwrap();
    assert_eq!(e.plugin_host().registry().tools().count(), Tool::ALL.len());
    e.tool = Tool::Box;
    assert!(!e.apply_at([1; 3]).unwrap());
    assert!(e.apply_at([2; 3]).unwrap());
    e.undo().unwrap();
    assert_eq!(e.project().to_bytes().unwrap(), before);
}

#[test]
fn absent_tool_bindings_are_inert_without_a_fallback_brush() {
    let mut e = Editor::blank().unwrap();
    e.plugins = RefCell::new(Host::default());
    for tool in Tool::ALL {
        e.tool = tool;
        assert!(!e.apply_at([2; 3]).unwrap());
        assert!(!e.click(front_ray(), true).unwrap());
        assert_eq!(e.tool_preview(Some([2; 3])), None);
    }
    assert_eq!(e.engine.revision(), 0);
    assert!(!e.engine.is_dirty());
}

#[test]
fn editor_uses_registered_behavior_for_commands_preview_and_sampling() {
    use voxelite_plugin_api::{Manifest, Plugin, Registrar, ToolContribution};
    struct RegisteredTool;
    impl Plugin for RegisteredTool {
        fn manifest(&self) -> Manifest {
            voxelite_plugin_tools::ToolsPlugin.manifest()
        }
        fn contribute(&self, registrar: &mut Registrar) {
            registrar.tool(ToolContribution {
                id: Tool::Place.id().into(),
                label_key: "Test".into(),
                icon: None,
            });
        }
        fn tool(&self, id: &str) -> Option<&dyn voxelite_plugin_api::Tool> {
            (id == Tool::Place.id()).then_some(self)
        }
    }
    impl voxelite_plugin_api::Tool for RegisteredTool {
        fn interact(
            &self,
            cx: &dyn ToolCx,
            _: ToolInput,
        ) -> std::result::Result<ToolOutcome, ToolError> {
            Ok(ToolOutcome {
                commands: vec![Command::SetVoxels {
                    layer: cx.active_layer(),
                    voxels: vec![
                        Voxel {
                            at: [5; 3],
                            slot: 1,
                        },
                        Voxel {
                            at: [6; 3],
                            slot: 1,
                        },
                    ],
                    mode: Mode::Place,
                    mirror: Mirror::default(),
                }],
                label: "Registered brush".into(),
                preview: Some(Region::new([5; 3], [7; 3]).unwrap()),
                target: Some([5; 3]),
                sampled_color: Some(Swatch::rgb(1, 2, 3)),
                anchor: None,
            })
        }
    }
    let mut e = Editor::blank().unwrap();
    e.plugins = RefCell::new(Host::new(vec![Box::new(RegisteredTool)]));
    let before = e.project().to_bytes().unwrap();
    assert_eq!(e.target(front_ray()), Some([5; 3]));
    assert_eq!(
        e.tool_preview(None),
        Some(Region::new([5; 3], [7; 3]).unwrap())
    );
    assert!(e.click(front_ray(), false).unwrap());
    assert_eq!(e.snapshot().layers[0].voxels.len(), 2);
    assert_eq!(e.color, Swatch::rgb(1, 2, 3));
    assert_eq!(e.engine.history().next().unwrap().label, "Registered brush");
    assert_eq!(e.engine.history().count(), 1);
    e.undo().unwrap();
    assert_eq!(e.project().to_bytes().unwrap(), before);
}
