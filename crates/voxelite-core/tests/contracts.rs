//! Executable core contracts and randomized history invariants.
use proptest::prelude::*;
use voxelite_core::*;
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn engine() -> Engine {
    Engine::new(Document::new(id(1), id(2))).unwrap()
}
fn tx(e: &Engine, n: u128, c: Command) -> Transaction {
    Transaction {
        expected_revision: e.revision(),
        id: id(n),
        actor: Actor::new(if n.is_multiple_of(2) {
            Origin::Human
        } else {
            Origin::Agent
        }),
        label: "Test edit".into(),
        command: c,
    }
}
fn set(p: Pos, slot: u16) -> Command {
    Command::SetVoxels {
        layer: id(2),
        voxels: vec![Voxel { at: p, slot }],
        mode: Mode::Place,
        mirror: Mirror::default(),
    }
}
fn run(e: &mut Engine, n: u128, c: Command) -> Receipt {
    e.execute(tx(e, n, c)).unwrap()
}
fn box_cmd(region: Region, hollow: bool, mode: Mode) -> Command {
    Command::Primitive {
        layer: id(2),
        shape: Shape::Box { region, hollow },
        mode,
        slot: if mode == Mode::Erase { None } else { Some(1) },
        mirror: Mirror::default(),
    }
}
#[test]
fn defaults_and_bounds() {
    let e = engine();
    assert_eq!(e.revision(), 0);
    assert_eq!(
        e.document().content.palette.get(1).unwrap().rgba,
        [180, 140, 100, 255]
    );
    assert!(Region::new([0; 3], [257, 1, 1]).is_err());
    assert!(Region::new([i32::MIN; 3], [i32::MAX; 3]).is_err());
    assert!(Region::new([32767; 3], [32768; 3]).is_ok());
}
#[test]
fn negative_chunk_coordinates() {
    assert_eq!(chunk_address([-1, -16, -17]), ([-1, -1, -2], 3855));
    let mut e = engine();
    run(&mut e, 3, set([-17, 0, -1], 1));
    assert_eq!(e.document().content.layers[0].voxels.get([-17, 0, -1]), 1);
}
#[test]
fn dense_promotion_demotion_and_empty_cleanup() {
    let mut e = engine();
    run(
        &mut e,
        3,
        box_cmd(
            Region::new([0; 3], [16, 16, 1]).unwrap(),
            false,
            Mode::Place,
        ),
    );
    assert!(
        e.document().content.layers[0]
            .voxels
            .chunk([0; 3])
            .unwrap()
            .is_dense()
    );
    run(
        &mut e,
        4,
        Command::EraseRegion {
            layer: id(2),
            region: Region::new([0; 3], [16, 9, 1]).unwrap(),
            mirror: Mirror::default(),
        },
    );
    assert!(
        !e.document().content.layers[0]
            .voxels
            .chunk([0; 3])
            .unwrap()
            .is_dense()
    );
    run(
        &mut e,
        5,
        Command::EraseRegion {
            layer: id(2),
            region: Region::new([0; 3], [16, 16, 1]).unwrap(),
            mirror: Mirror::default(),
        },
    );
    assert_eq!(e.document().content.layers[0].voxels.chunk_count(), 0);
}
#[test]
fn invalid_batch_rolls_back_palette_geometry_revision_history() {
    let mut e = engine();
    let before = e.snapshot();
    let c = Command::Batch(vec![
        set([0; 3], 1),
        Command::Palette(PaletteEdit::Add {
            slot: Some(2),
            color: Swatch::rgb(3, 4, 5),
        }),
        set([99; 3], 2),
    ]);
    let error = e.execute(tx(&e, 3, c)).unwrap_err();
    assert!(matches!(error, Error::Batch { index: 2, .. }));
    assert_eq!(e.snapshot(), before);
    assert_eq!(e.revision(), 0);
    assert_eq!(e.history().count(), 0);
}
#[test]
fn noops_do_not_change_revision_or_clear_redo() {
    let mut e = engine();
    run(&mut e, 3, set([0; 3], 1));
    e.undo(1, id(3)).unwrap();
    let r = run(&mut e, 4, set([0; 3], 0));
    assert_eq!(r.entry, None);
    assert_eq!(e.revision(), 2);
    assert_eq!(e.redo_head(), Some(id(3)));
}
#[test]
fn revision_and_head_guards_shared_history_and_dirty() {
    let mut e = engine();
    run(&mut e, 3, set([0; 3], 1));
    assert_eq!(e.history().next().unwrap().actor.origin, Origin::Agent);
    assert!(e.undo(0, id(3)).is_err());
    assert!(e.undo(1, id(9)).is_err());
    assert!(e.is_dirty());
    e.undo(1, id(3)).unwrap();
    assert!(!e.is_dirty());
    e.redo(2, id(3)).unwrap();
    assert_eq!(e.revision(), 3);
    e.mark_saved();
    run(&mut e, 4, set([1, 0, 0], 1));
    e.undo(4, id(4)).unwrap();
    assert!(!e.is_dirty());
    run(&mut e, 5, set([2, 0, 0], 1));
    assert!(e.redo_head().is_none());
}
#[test]
fn lock_visibility_and_layer_order() {
    let mut e = engine();
    run(
        &mut e,
        3,
        Command::Batch(vec![
            set([0; 3], 1),
            Command::Palette(PaletteEdit::Add {
                slot: Some(2),
                color: Swatch::rgb(9, 8, 7),
            }),
            Command::Layer(LayerEdit::Create {
                id: id(10),
                name: "Top".into(),
                index: 1,
            }),
            Command::SetVoxels {
                layer: id(10),
                voxels: vec![Voxel {
                    at: [0; 3],
                    slot: 2,
                }],
                mode: Mode::Place,
                mirror: Mirror::default(),
            },
        ]),
    );
    assert_eq!(e.document().content.sample([0; 3]), Some((2, id(10))));
    run(
        &mut e,
        4,
        Command::Layer(LayerEdit::Update {
            id: id(10),
            name: None,
            visible: Some(false),
            locked: Some(true),
        }),
    );
    assert_eq!(e.document().content.sample([0; 3]), Some((1, id(2))));
    assert!(
        e.execute(tx(&e, 5, Command::Layer(LayerEdit::Delete(id(10)))))
            .is_err()
    );
    run(
        &mut e,
        6,
        Command::Layer(LayerEdit::Update {
            id: id(2),
            name: None,
            visible: Some(false),
            locked: None,
        }),
    );
    assert!(e.execute(tx(&e, 7, set([1, 0, 0], 1))).is_err());
}
#[test]
fn mirror_plane2_duplicates_and_conflicts() {
    let mut e = engine();
    let m = Mirror {
        plane2: [Some(0), None, None],
    };
    run(
        &mut e,
        3,
        Command::SetVoxels {
            layer: id(2),
            voxels: vec![
                Voxel {
                    at: [0; 3],
                    slot: 0,
                },
                Voxel {
                    at: [0; 3],
                    slot: 1,
                },
            ],
            mode: Mode::Place,
            mirror: m,
        },
    );
    assert_eq!(e.document().content.layers[0].voxels.len(), 2);
    assert_eq!(e.document().content.layers[0].voxels.get([-1, 0, 0]), 1);
    let before = e.snapshot();
    assert!(
        e.execute(tx(
            &e,
            4,
            Command::SetVoxels {
                layer: id(2),
                voxels: vec![
                    Voxel {
                        at: [0; 3],
                        slot: 1
                    },
                    Voxel {
                        at: [-1, 0, 0],
                        slot: 0
                    }
                ],
                mode: Mode::Place,
                mirror: m
            }
        ))
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
    run(
        &mut e,
        5,
        Command::SetVoxels {
            layer: id(2),
            voxels: vec![Voxel {
                at: [2, 0, 0],
                slot: 1,
            }],
            mode: Mode::Place,
            mirror: Mirror {
                plane2: [Some(5), None, None],
            },
        },
    );
    assert_eq!(e.document().content.layers[0].voxels.len(), 3);
}
#[test]
fn mirror_out_of_bounds_atomic() {
    let mut e = engine();
    let before = e.snapshot();
    assert!(
        e.execute(tx(
            &e,
            3,
            Command::SetVoxels {
                layer: id(2),
                voxels: vec![Voxel {
                    at: [0; 3],
                    slot: 1
                }],
                mode: Mode::Place,
                mirror: Mirror {
                    plane2: [None, Some(0), None]
                }
            }
        ))
        .is_err()
    );
    assert_eq!(e.snapshot(), before);
}
#[test]
fn fill_is_six_connected_and_capped() {
    let mut e = engine();
    run(
        &mut e,
        3,
        Command::Batch(vec![
            set([0; 3], 1),
            set([1, 1, 0], 1),
            Command::Palette(PaletteEdit::Add {
                slot: Some(2),
                color: Swatch::rgb(2, 3, 4),
            }),
        ]),
    );
    let c = Command::FloodFill {
        layer: id(2),
        seed: [0; 3],
        slot: 2,
        region: Some(Region::new([0; 3], [3; 3]).unwrap()),
        cap: 27,
        mirror: Mirror::default(),
    };
    assert_eq!(run(&mut e, 4, c).changes.recolored, 1);
    assert_eq!(e.document().content.layers[0].voxels.get([1, 1, 0]), 1);
    let before = e.snapshot();
    let c = Command::FloodFill {
        layer: id(2),
        seed: [2; 3],
        slot: 2,
        region: Some(Region::new([0; 3], [3; 3]).unwrap()),
        cap: 4,
        mirror: Mirror::default(),
    };
    assert!(e.execute(tx(&e, 5, c)).is_err());
    assert_eq!(e.snapshot(), before);
}
#[test]
fn hollow_box_paint_and_erase() {
    let mut e = engine();
    let r = Region::new([0; 3], [3; 3]).unwrap();
    assert_eq!(
        run(&mut e, 3, box_cmd(r, true, Mode::Place)).changes.added,
        26
    );
    assert_eq!(e.document().content.layers[0].voxels.get([1; 3]), 0);
    assert_eq!(run(&mut e, 4, box_cmd(r, false, Mode::Paint)).entry, None);
    assert_eq!(
        run(&mut e, 5, box_cmd(r, true, Mode::Erase))
            .changes
            .removed,
        26
    );
}
#[test]
fn shape_boundaries_and_line_ties() {
    let mut e = engine();
    run(
        &mut e,
        3,
        Command::Primitive {
            layer: id(2),
            shape: Shape::Ellipsoid {
                center2: [3; 3],
                radii2: [2; 3],
                hollow: false,
            },
            mode: Mode::Place,
            slot: Some(1),
            mirror: Mirror::default(),
        },
    );
    assert_eq!(e.document().content.layers[0].voxels.len(), 7);
    let mut e = engine();
    run(
        &mut e,
        3,
        Command::Primitive {
            layer: id(2),
            shape: Shape::Line {
                from: [0; 3],
                to: [1; 3],
            },
            mode: Mode::Place,
            slot: Some(1),
            mirror: Mirror::default(),
        },
    );
    assert_eq!(e.document().content.layers[0].voxels.len(), 8);
}
#[test]
fn overlapping_move_uses_source_snapshot() {
    let mut e = engine();
    run(
        &mut e,
        3,
        box_cmd(Region::new([0; 3], [3, 1, 1]).unwrap(), false, Mode::Place),
    );
    run(
        &mut e,
        4,
        Command::TransformRegion(Transform {
            source: id(2),
            destination: id(2),
            region: Region::new([0; 3], [3, 1, 1]).unwrap(),
            move_cells: true,
            translate: [1, 0, 0],
            pivot2: [0; 3],
            reflect: [false; 3],
            rotate: None,
            collision: Collision::Reject,
        }),
    );
    assert_eq!(
        e.document().content.layers[0].voxels.sorted(),
        vec![([1, 0, 0], 1), ([2, 0, 0], 1), ([3, 0, 0], 1)]
    );
}
#[test]
fn palette_reference_remap_and_order() {
    let mut e = engine();
    run(
        &mut e,
        3,
        Command::Batch(vec![
            set([0; 3], 1),
            Command::Palette(PaletteEdit::Add {
                slot: Some(256),
                color: Swatch::rgb(1, 2, 3),
            }),
        ]),
    );
    assert!(
        e.execute(tx(
            &e,
            4,
            Command::Palette(PaletteEdit::Remove {
                slot: 1,
                replacement: None
            })
        ))
        .is_err()
    );
    run(
        &mut e,
        5,
        Command::Palette(PaletteEdit::Reorder(vec![256, 1])),
    );
    assert_eq!(e.document().content.layers[0].voxels.get([0; 3]), 1);
    run(
        &mut e,
        6,
        Command::Palette(PaletteEdit::Remove {
            slot: 1,
            replacement: Some(256),
        }),
    );
    assert_eq!(e.document().content.layers[0].voxels.get([0; 3]), 256);
}
#[test]
fn comments_versions_survive_geometry_and_restore() {
    let mut e = engine();
    run(&mut e, 3, set([0; 3], 1));
    run(
        &mut e,
        4,
        Command::CreateComment {
            id: id(20),
            anchor: Anchor::Voxel { at: [0; 3] },
            layer: Some(id(2)),
            message: id(21),
            text: "Check silhouette".into(),
        },
    );
    run(
        &mut e,
        5,
        Command::Reply {
            id: id(20),
            message: id(22),
            text: "Looks good".into(),
        },
    );
    assert_eq!(
        e.document().content.comments[0].messages[1].origin,
        Origin::Agent
    );
    run(
        &mut e,
        6,
        Command::SaveVersion {
            id: id(30),
            name: "Blockout".into(),
            parent: None,
        },
    );
    run(&mut e, 7, set([0; 3], 0));
    assert_eq!(
        e.document()
            .content
            .anchor_state(&e.document().content.comments[0]),
        AnchorState::Orphaned
    );
    run(
        &mut e,
        8,
        Command::Resolve {
            id: id(20),
            resolved: true,
        },
    );
    let head = e.head();
    run(
        &mut e,
        9,
        Command::RestoreVersion {
            id: id(30),
            expected_head: head,
        },
    );
    assert!(!e.document().content.comments[0].resolved);
    assert_eq!(e.versions().count(), 1);
    let (branch, map) = e.branch(id(30), id(40), "Alternative".into()).unwrap();
    assert_eq!(map[&id(2)], branch.content.layers[0].id);
    assert_eq!(
        branch.content.comments[0].layer,
        Some(branch.content.layers[0].id)
    );
    assert!(branch.versions.is_empty());
    assert_eq!(
        e.versions().next().unwrap().content_hash,
        e.versions()
            .next()
            .unwrap()
            .snapshot
            .content_hash()
            .unwrap()
    );
}
#[test]
fn previews_checkpoints_and_conflict_aware_revert() {
    let mut e = engine();
    e.checkpoint(id(50), "Start".into()).unwrap();
    run(&mut e, 3, set([0; 3], 1));
    run(&mut e, 4, set([1, 0, 0], 1));
    let revision = e.revision();
    assert_eq!(
        e.preview(Some(id(3))).unwrap().content.layers[0]
            .voxels
            .len(),
        1
    );
    assert_eq!(e.revision(), revision);
    e.revert(revision, id(3), id(5), Actor::new(Origin::Human))
        .unwrap();
    assert_eq!(
        e.document().content.layers[0].voxels.sorted(),
        vec![([1, 0, 0], 1)]
    );
    e.restore_checkpoint(
        e.revision(),
        e.head(),
        id(50),
        id(6),
        Actor::new(Origin::Human),
    )
    .unwrap();
    assert!(e.document().content.layers[0].voxels.is_empty());
}
#[test]
fn history_budget_rejects_before_mutation_and_evicts_complete_entries() {
    let d = Document::new(id(1), id(2));
    let mut small = Engine::with_limits(
        d.clone(),
        Limits {
            history_bytes: 1,
            work_units: MAX_WORK,
        },
    )
    .unwrap();
    assert!(small.execute(tx(&small, 3, set([0; 3], 1))).is_err());
    assert_eq!(small.revision(), 0);
    let mut e = Engine::with_limits(
        d,
        Limits {
            history_bytes: 6000,
            work_units: MAX_WORK,
        },
    )
    .unwrap();
    for n in 3..10 {
        run(&mut e, n, set([n as i32, 0, 0], 1));
    }
    assert!(e.history_bytes() <= 6000);
    assert!(!e.initial_retained());
    assert!(e.preview(None).is_err());
}
#[test]
fn work_is_summed_and_validate_does_not_commit() {
    let mut e = Engine::with_limits(
        Document::new(id(1), id(2)),
        Limits {
            work_units: 3,
            ..Limits::default()
        },
    )
    .unwrap();
    let c = Command::Batch(vec![
        set([0; 3], 1),
        set([1, 0, 0], 1),
        set([2, 0, 0], 1),
        set([3, 0, 0], 1),
    ]);
    assert!(e.execute(tx(&e, 3, c)).is_err());
    assert_eq!(e.revision(), 0);
    let r = e.validate(&tx(&e, 4, set([0; 3], 1))).unwrap();
    assert_eq!(r.changes.added, 1);
    assert_eq!(e.revision(), 0);
}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]
    #[test]
    fn randomized_apply_undo_redo(ops in prop::collection::vec((-20i32..20,0i32..20,-20i32..20,any::<bool>()),1..12)){
        let mut e=engine();let original=e.snapshot();let mut states=Vec::new();let mut oracle=std::collections::BTreeMap::new();
        for (i,(x,y,z,occupied)) in ops.into_iter().enumerate(){let r=run(&mut e,(i+100)as u128,set([x,y,z],u16::from(occupied)));if occupied{oracle.insert([x,y,z],1);}else{oracle.remove(&[x,y,z]);}prop_assert_eq!(e.document().content.layers[0].voxels.sorted(),oracle.iter().map(|(p,s)|(*p,*s)).collect::<Vec<_>>());if r.entry.is_some(){states.push(e.snapshot());}}
        let final_state=e.snapshot();while let Some(h)=e.head(){e.undo(e.revision(),h).unwrap();}prop_assert_eq!(e.snapshot(),original);
        for state in states{let h=e.redo_head().unwrap();e.redo(e.revision(),h).unwrap();prop_assert_eq!(e.snapshot(),state);}prop_assert_eq!(e.snapshot(),final_state);
    }
}
