use super::*;
use quantum_plugin_api::{CoreError, Palette, PaletteSlot, Swatch, ToolHit, ToolLayer, Uuid};
use std::collections::BTreeMap;

struct FakeCx {
    hit: Option<ToolHit>,
    ground: Option<Pos>,
    voxels: BTreeMap<Pos, u16>,
    palette: Palette,
    color: Swatch,
    layer: ToolLayer,
}
impl Default for FakeCx {
    fn default() -> Self {
        Self {
            hit: Some(ToolHit {
                at: [2; 3],
                normal: [0, 0, 1],
                layer: Uuid::nil(),
                slot: 1,
            }),
            ground: Some([4, 0, 4]),
            voxels: [([2; 3], 1)].into(),
            palette: Palette::default(),
            color: Swatch::rgb(17, 34, 51),
            layer: ToolLayer {
                visible: true,
                locked: false,
            },
        }
    }
}
impl ToolCx for FakeCx {
    fn hovered_hit(&self) -> Option<ToolHit> {
        self.hit
    }
    fn ground_target(&self) -> Option<Pos> {
        self.ground
    }
    fn sample(&self, at: Pos) -> Option<(u16, Uuid)> {
        self.voxels.get(&at).map(|slot| (*slot, Uuid::nil()))
    }
    fn voxel(&self, layer: Uuid, at: Pos) -> Result<u16, CoreError> {
        self.layer(layer)?;
        Ok(self.voxels.get(&at).copied().unwrap_or(0))
    }
    fn palette(&self) -> &Palette {
        &self.palette
    }
    fn active_color(&self) -> &Swatch {
        &self.color
    }
    fn active_layer(&self) -> Uuid {
        Uuid::nil()
    }
    fn layer(&self, id: Uuid) -> Result<ToolLayer, CoreError> {
        if id == Uuid::nil() {
            Ok(self.layer)
        } else {
            Err(CoreError::NotFound("layer".into()))
        }
    }
    fn bounds(&self) -> Region {
        Region::new([0; 3], [32; 3]).unwrap()
    }
}

fn input(gesture: ToolGesture) -> ToolInput {
    ToolInput {
        gesture,
        anchor: None,
    }
}

#[test]
fn clicks_propose_exact_unmirrored_commands_and_labels() {
    let cx = FakeCx::default();
    for (tool, at, mode, slot) in [
        (BuiltinTool::Place, [2, 2, 3], Mode::Place, 2),
        (BuiltinTool::Erase, [2; 3], Mode::Erase, 0),
        (BuiltinTool::Paint, [2; 3], Mode::Paint, 2),
        (BuiltinTool::Fill, [2; 3], Mode::Paint, 2),
    ] {
        let outcome = tool.interact(&cx, input(ToolGesture::Click)).unwrap();
        let mut commands = Vec::new();
        if slot != 0 {
            commands.push(Command::Palette(PaletteEdit::Add {
                slot: Some(2),
                color: cx.color.clone(),
            }));
        }
        commands.push(if tool == BuiltinTool::Fill {
            Command::FloodFill {
                layer: Uuid::nil(),
                seed: at,
                slot,
                region: None,
                cap: 1_048_576,
                mirror: Mirror::default(),
            }
        } else {
            Command::SetVoxels {
                layer: Uuid::nil(),
                voxels: vec![Voxel { at, slot }],
                mode,
                mirror: Mirror::default(),
            }
        });
        assert_eq!(outcome.commands, commands, "{tool:?}");
        assert_eq!(outcome.label, tool.label());
        assert_eq!(outcome.target, Some(at));
        assert_eq!(
            outcome.preview,
            Some(Region::new(at, at.map(|v| v + 1)).unwrap())
        );
        assert_eq!(outcome.anchor, None);
        assert_eq!(outcome.sampled_color, None);
    }
    // Tools only proposed edits; the fake document and palette remain untouched.
    assert_eq!(cx.voxels, [([2; 3], 1)].into());
    assert_eq!(cx.palette, Palette::default());
}

#[test]
fn box_tracks_corners_and_preview_without_editing_until_second_click() {
    let mut cx = FakeCx::default();
    let first = BuiltinTool::Box
        .interact(&cx, input(ToolGesture::Click))
        .unwrap();
    assert_eq!(first.anchor, Some([2, 2, 3]));
    assert!(first.commands.is_empty());
    cx.hit = None;
    let region = Region::new([2, 0, 3], [5, 3, 5]).unwrap();
    let mut gesture = ToolInput {
        gesture: ToolGesture::Hover,
        anchor: first.anchor,
    };
    let hover = BuiltinTool::Box.interact(&cx, gesture).unwrap();
    assert_eq!(hover.preview, Some(region));
    assert!(hover.commands.is_empty());
    gesture.gesture = ToolGesture::Preview(None);
    assert_eq!(
        BuiltinTool::Box.interact(&cx, gesture).unwrap().preview,
        first.preview
    );
    gesture.gesture = ToolGesture::Click;
    let committed = BuiltinTool::Box.interact(&cx, gesture).unwrap();
    assert_eq!(
        committed.commands,
        vec![
            Command::Palette(PaletteEdit::Add {
                slot: Some(2),
                color: cx.color
            }),
            Command::Primitive {
                layer: Uuid::nil(),
                shape: Shape::Box {
                    region,
                    hollow: false
                },
                mode: Mode::Place,
                slot: Some(2),
                mirror: Mirror::default()
            },
        ]
    );
    assert_eq!(committed.label, "Box");
    assert_eq!(committed.anchor, None);
    assert_eq!(
        committed.preview,
        Some(Region::new([4, 0, 4], [5, 1, 5]).unwrap())
    );
}

#[test]
fn hover_targets_faces_or_ground_and_never_proposes_commands() {
    let mut cx = FakeCx::default();
    for tool in BuiltinTool::ALL {
        let adds = matches!(tool, BuiltinTool::Place | BuiltinTool::Box);
        let at = if adds { [2, 2, 3] } else { [2; 3] };
        let hover = tool.interact(&cx, input(ToolGesture::Hover)).unwrap();
        assert_eq!(hover.target, Some(at));
        assert!(hover.commands.is_empty());
        assert_eq!(hover.sampled_color, None);
    }
    cx.hit = None;
    for tool in BuiltinTool::ALL {
        let adds = matches!(tool, BuiltinTool::Place | BuiltinTool::Box);
        let hover = tool.interact(&cx, input(ToolGesture::Hover)).unwrap();
        assert_eq!(hover.target, if adds { cx.ground } else { None });
    }
    cx.hit = Some(ToolHit {
        at: [2, 2, 31],
        normal: [0, 0, 1],
        layer: Uuid::nil(),
        slot: 1,
    });
    for tool in [BuiltinTool::Place, BuiltinTool::Box] {
        assert_eq!(
            tool.interact(&cx, input(ToolGesture::Click))
                .unwrap()
                .target,
            None
        );
    }
}

#[test]
fn picker_reads_original_visible_color_and_retains_pending_anchor() {
    let mut cx = FakeCx {
        layer: ToolLayer {
            visible: false,
            locked: true,
        },
        ..Default::default()
    };
    cx.hit.as_mut().unwrap().layer = Uuid::from_u128(1);
    for gesture in [ToolGesture::Click, ToolGesture::ApplyAt([2; 3])] {
        let outcome = BuiltinTool::Picker
            .interact(
                &cx,
                ToolInput {
                    gesture,
                    anchor: Some([1; 3]),
                },
            )
            .unwrap();
        assert_eq!(outcome.sampled_color, cx.palette.get(1).cloned());
        assert!(outcome.commands.is_empty());
        assert_eq!(outcome.anchor, Some([1; 3]));
    }
    cx.hit = None;
    assert_eq!(
        BuiltinTool::Picker
            .interact(&cx, input(ToolGesture::Click))
            .unwrap()
            .sampled_color,
        None
    );
    assert_eq!(
        BuiltinTool::Picker
            .interact(&cx, input(ToolGesture::ApplyAt([3; 3])))
            .unwrap()
            .sampled_color,
        None
    );
}

#[test]
fn air_is_a_noop_and_palette_matching_uses_rgba_and_lowest_free_slot() {
    let mut cx = FakeCx::default();
    for tool in [BuiltinTool::Paint, BuiltinTool::Erase, BuiltinTool::Fill] {
        assert!(
            tool.interact(&cx, input(ToolGesture::ApplyAt([3; 3])))
                .unwrap()
                .commands
                .is_empty()
        );
    }
    cx.color = cx.palette.get(1).unwrap().clone();
    cx.color.name = Some("different name".into());
    for tool in [BuiltinTool::Place, BuiltinTool::Paint, BuiltinTool::Fill] {
        let outcome = tool
            .interact(&cx, input(ToolGesture::ApplyAt([2; 3])))
            .unwrap();
        assert_eq!(outcome.commands.len(), 1);
        assert!(!matches!(outcome.commands[0], Command::Palette(_)));
    }
    cx.palette.slots.push(PaletteSlot {
        index: 3,
        color: Swatch::rgb(1, 2, 3),
    });
    cx.palette.order = vec![3, 1];
    cx.color = Swatch::rgb(17, 34, 51);
    assert!(matches!(
        BuiltinTool::Place
            .interact(&cx, input(ToolGesture::Click))
            .unwrap()
            .commands[0],
        Command::Palette(PaletteEdit::Add { slot: Some(2), .. })
    ));
}

#[test]
fn layer_and_bounds_validation_matches_editor_gestures() {
    let mut cx = FakeCx::default();
    for layer in [
        ToolLayer {
            visible: false,
            locked: false,
        },
        ToolLayer {
            visible: true,
            locked: true,
        },
    ] {
        cx.layer = layer;
        for tool in BuiltinTool::ALL {
            assert_eq!(
                tool.interact(&cx, input(ToolGesture::ApplyAt([i32::MAX; 3])))
                    .unwrap_err()
                    .to_string(),
                "Outside document bounds"
            );
            assert_eq!(
                tool.interact(&cx, input(ToolGesture::ApplyAt([32, 0, 0])))
                    .unwrap_err()
                    .to_string(),
                "Outside document bounds"
            );
            if tool != BuiltinTool::Picker {
                assert_eq!(
                    tool.interact(&cx, input(ToolGesture::ApplyAt([2; 3])))
                        .unwrap_err()
                        .to_string(),
                    "The active layer is locked or hidden"
                );
            }
        }
    }
    cx.hit.as_mut().unwrap().layer = Uuid::from_u128(1);
    for tool in [BuiltinTool::Paint, BuiltinTool::Erase, BuiltinTool::Fill] {
        assert_eq!(
            tool.interact(&cx, input(ToolGesture::Click))
                .unwrap_err()
                .to_string(),
            "Select the hit voxel’s layer to edit it"
        );
    }
}

#[test]
fn full_palette_rejects_new_colors_but_allows_erase_and_first_box_corner() {
    let mut cx = FakeCx::default();
    cx.palette.slots = (1..=256)
        .map(|index| PaletteSlot {
            index,
            color: Swatch::rgb(0, (index - 1) as u8, 0),
        })
        .collect();
    cx.palette.order = (1..=256).collect();
    for tool in [
        BuiltinTool::Place,
        BuiltinTool::Paint,
        BuiltinTool::Fill,
        BuiltinTool::Box,
    ] {
        let gesture = ToolInput {
            gesture: ToolGesture::ApplyAt([2; 3]),
            anchor: Some([1; 3]),
        };
        assert_eq!(
            tool.interact(&cx, gesture).unwrap_err().to_string(),
            "Palette is full (256 colors)"
        );
    }
    assert_eq!(
        BuiltinTool::Erase
            .interact(&cx, input(ToolGesture::Click))
            .unwrap()
            .commands
            .len(),
        1
    );
    assert!(
        BuiltinTool::Box
            .interact(&cx, input(ToolGesture::Click))
            .unwrap()
            .commands
            .is_empty()
    );
}
