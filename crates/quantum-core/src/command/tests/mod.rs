use crate::*;

mod properties;
mod strategies;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn transaction(e: &Engine, serial: u128, command: Command) -> Transaction {
    Transaction {
        expected_revision: e.revision(),
        id: id(100 + serial),
        actor: Actor {
            origin: if serial.is_multiple_of(2) {
                Origin::Human
            } else {
                Origin::Agent
            },
            client_id: id(50 + serial % 2),
            tool: "test".into(),
        },
        label: "Edit".into(),
        command,
    }
}

fn rename(name: &str) -> Command {
    Command::EditDocument {
        name: Some(name.into()),
        bounds: None,
    }
}

fn write(voxels: Vec<Voxel>, mirror: Mirror) -> Command {
    Command::SetVoxels {
        layer: id(2),
        voxels,
        mode: Mode::Place,
        mirror,
    }
}

fn execute(e: &mut Engine, serial: u128, command: Command) -> Receipt {
    e.execute(transaction(e, serial, command)).unwrap()
}

#[test]
fn net_noop_batch_preserves_redo_and_a_new_edit_discards_it() {
    let mut e = Engine::new(Document::new(id(1), id(2))).unwrap();
    let original = e.snapshot();
    let receipt = execute(&mut e, 0, rename("First"));
    e.undo(e.revision(), receipt.entry.unwrap()).unwrap();
    let bytes = e.history_bytes();
    let revision = e.revision();
    let noop = execute(
        &mut e,
        1,
        Command::Batch(vec![rename("Temporary"), rename(&original.content.name)]),
    );
    assert_eq!(noop.entry, None);
    assert_eq!(e.snapshot(), original);
    assert_eq!(e.revision(), revision);
    assert_eq!(e.redo_head(), receipt.entry);
    assert_eq!(e.history_bytes(), bytes);
    execute(&mut e, 2, rename("Branch"));
    assert_eq!(e.redo_head(), None);
    assert_eq!(e.timeline().count(), 1);
    assert!(e.preview(receipt.entry).is_err());
}

#[test]
fn duplicate_writes_keep_the_last_slot_before_mirror_expansion() {
    let mut e = Engine::new(Document::new(id(1), id(2))).unwrap();
    // The discarded undefined slot must not be validated or mirrored.
    let receipt = execute(
        &mut e,
        0,
        write(
            vec![
                Voxel {
                    at: [0; 3],
                    slot: 256,
                },
                Voxel {
                    at: [0; 3],
                    slot: 1,
                },
            ],
            Mirror {
                plane2: [Some(0), None, None],
            },
        ),
    );
    assert_eq!(receipt.work_units, 3);
    assert_eq!(receipt.changes.added, 2);
    let voxels = &e.document().content.layers[0].voxels;
    assert_eq!(voxels.get([0; 3]), 1);
    assert_eq!(voxels.get([-1, 0, 0]), 1);
}

#[test]
fn mirror_candidates_are_charged_before_deduplication() {
    let doc = Document::new(id(1), id(2));
    for limit in [7, 8] {
        let mut e = Engine::with_limits(
            doc.clone(),
            Limits {
                work_units: limit,
                ..Limits::default()
            },
        )
        .unwrap();
        let command = write(
            vec![Voxel {
                at: [0; 3],
                slot: 1,
            }],
            Mirror {
                plane2: [Some(1); 3],
            },
        );
        let result = e.execute(transaction(&e, 0, command));
        if limit == 7 {
            assert!(matches!(result, Err(Error::Limit(_))));
            assert_eq!(e.document(), &doc);
            assert_eq!(e.revision(), 0);
            assert_eq!(e.history_bytes(), 0);
        } else {
            let receipt = result.unwrap();
            assert_eq!(receipt.work_units, 8);
            assert_eq!(receipt.changes.added, 1);
        }
    }
}

#[test]
fn work_budget_checks_the_exact_limit_and_integer_overflow() {
    let mut work = super::Work { used: 0, limit: 2 };
    work.charge(2).unwrap();
    assert!(matches!(work.charge(1), Err(Error::Limit(_))));
    let mut work = super::Work {
        used: usize::MAX,
        limit: usize::MAX,
    };
    assert!(matches!(work.charge(1), Err(Error::Limit(_))));
}

#[test]
fn a_late_palette_remap_failure_leaves_every_layer_and_slot_intact() {
    let mut doc = Document::new(id(1), id(2));
    doc.content.palette =
        Palette::from_colors(vec![Swatch::rgb(1, 2, 3), Swatch::rgb(4, 5, 6)]).unwrap();
    doc.content.layers[0].voxels.set([0; 3], 1);
    let mut locked = Layer::new(id(3), "Locked");
    locked.locked = true;
    locked.voxels.set([1, 0, 0], 1);
    doc.content.layers.push(locked);
    let mut e = Engine::new(doc.clone()).unwrap();
    let command = Command::Palette(PaletteEdit::Remove {
        slot: 1,
        replacement: Some(2),
    });
    assert_eq!(
        e.execute(transaction(&e, 0, command)),
        Err(Error::LayerBlocked(id(3)))
    );
    assert_eq!(e.document(), &doc);
    assert_eq!(e.revision(), 0);
    assert_eq!(e.history_bytes(), 0);
}

#[test]
fn batch_rejects_each_forbidden_family_at_its_index() {
    let forbidden = [
        Command::Batch(vec![]),
        Command::SaveVersion {
            id: id(3),
            name: "Saved".into(),
            parent: None,
        },
        Command::RestoreVersion {
            id: id(3),
            expected_head: None,
        },
        Command::CreateComment {
            id: id(3),
            anchor: Anchor::Voxel { at: [0; 3] },
            layer: None,
            message: id(4),
            text: "Text".into(),
        },
        Command::Reanchor {
            id: id(3),
            anchor: Anchor::Voxel { at: [0; 3] },
            layer: None,
        },
        Command::Reply {
            id: id(3),
            message: id(4),
            text: "Text".into(),
        },
        Command::Resolve {
            id: id(3),
            resolved: true,
        },
    ];
    for command in forbidden {
        let mut e = Engine::new(Document::new(id(1), id(2))).unwrap();
        let original = e.snapshot();
        let result = e.execute(transaction(
            &e,
            0,
            Command::Batch(vec![rename("Temporary"), command]),
        ));
        assert!(
            matches!(result, Err(Error::Batch { index: 1, source }) if *source == Error::Invalid("command not permitted in batch".into()))
        );
        assert_eq!(e.snapshot(), original);
        assert_eq!(e.revision(), 0);
        assert_eq!(e.head(), None);
    }
}

#[test]
fn batch_size_limit_is_inclusive_even_for_noops() {
    let mut e = Engine::new(Document::new(id(1), id(2))).unwrap();
    let noop = rename(&e.document().content.name);
    assert_eq!(
        execute(&mut e, 0, Command::Batch(vec![noop.clone(); 256])).entry,
        None
    );
    assert!(matches!(
        e.execute(transaction(&e, 1, Command::Batch(vec![noop; 257]))),
        Err(Error::Limit(_))
    ));
    assert_eq!(e.revision(), 0);
}

#[test]
fn command_voxels_keep_tuple_encoding_and_voxel_objects_keep_fields() {
    let voxel = Voxel {
        at: [-1, 2, 3],
        slot: 0,
    };
    let command = write(vec![voxel], Mirror::default());
    let value = serde_json::to_value(&command).unwrap();
    assert_eq!(
        value["set_voxels"]["voxels"],
        serde_json::json!([[-1, 2, 3, 0]])
    );
    assert_eq!(
        serde_json::to_value(voxel).unwrap(),
        serde_json::json!({"at": [-1, 2, 3], "slot": 0})
    );
    assert_eq!(
        serde_json::from_value::<Command>(value.clone()).unwrap(),
        command
    );
    for invalid in [-1, 65536] {
        let mut value = value.clone();
        value["set_voxels"]["voxels"][0][3] = serde_json::json!(invalid);
        assert!(serde_json::from_value::<Command>(value).is_err());
    }
}
