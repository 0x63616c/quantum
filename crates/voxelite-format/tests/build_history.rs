//! Portable build recipe validation, resource limits and compatibility.
use serde_json::{Value, json};
use voxelite_core::*;
use voxelite_format::*;

fn small_build() -> Project {
    let layer = Uuid::from_u128(2);
    let mut engine = Engine::new(Document::new(Uuid::from_u128(1), layer)).unwrap();
    let mut history = BuildHistory {
        version: 1,
        base_layer: layer,
        steps: vec![],
    };
    for (index, command) in [
        Command::EditDocument {
            name: Some("Small build".into()),
            bounds: None,
        },
        Command::SetVoxels {
            layer,
            voxels: vec![
                Voxel {
                    at: [0; 3],
                    slot: 1,
                },
                Voxel {
                    at: [1, 0, 0],
                    slot: 1,
                },
            ],
            mode: Mode::Place,
            mirror: Mirror::default(),
        },
        Command::Palette(PaletteEdit::Set {
            slot: 1,
            color: Swatch::rgb(80, 90, 100),
        }),
    ]
    .into_iter()
    .enumerate()
    {
        let step = BuildStep {
            id: Uuid::from_u128(10 + index as u128),
            label: format!("Step {index}"),
            origin: Origin::Agent,
            elapsed_ms: index as u32 * 60_000,
            command,
        };
        engine
            .execute(Transaction {
                expected_revision: engine.revision(),
                id: step.id,
                label: step.label.clone(),
                actor: Actor::new(step.origin),
                command: step.command.clone(),
            })
            .unwrap();
        history.steps.push(step);
    }
    let mut project = Project::new(engine.document().clone()).unwrap();
    project.set_build_history(history).unwrap();
    project
}

#[test]
fn rejects_tampering_noops_invalid_commands_and_noncanonical_fields() {
    let project = small_build();
    let original: Value = serde_json::from_slice(&project.to_bytes().unwrap()).unwrap();
    for (path, replacement) in [
        ("/build_history/version", json!(2)),
        ("/build_history/steps", json!([])),
        (
            "/build_history/steps/1/id",
            original["build_history"]["steps"][0]["id"].clone(),
        ),
        ("/build_history/steps/1/elapsed_ms", json!(0)),
        ("/build_history/steps/1/label", json!("")),
        (
            "/build_history/steps/1/command/set_voxels/voxels",
            json!([[10000, 0, 0, 1]]),
        ),
        (
            "/build_history/steps/1/command/set_voxels/voxels",
            json!([[0, 0, 0, 257]]),
        ),
        (
            "/build_history/steps/1/command/set_voxels/voxels",
            json!([[0, 0, 0, -1]]),
        ),
        (
            "/build_history/steps/1/command/set_voxels/voxels",
            json!([]),
        ),
        (
            "/build_history/steps/1/command/set_voxels/layer",
            json!("missing layer"),
        ),
        (
            "/build_history/steps/0/id",
            json!("00000000-0000-0000-0000-00000000000A"),
        ),
        ("/layers/0/voxels", json!([[1, 0, 0, 1]])),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(path).unwrap() = replacement;
        assert!(
            from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err(),
            "{path}"
        );
    }
    let mut changed = original.clone();
    changed["build_history"]["steps"][1]["command"]["set_voxels"]["unexpected"] = json!(true);
    assert!(from_bytes(&serde_json::to_vec(&changed).unwrap()).is_err());
    let mut history = project.build_history().unwrap().clone();
    history.steps = vec![history.steps[0].clone(); 65];
    let mut copy = project.clone();
    assert!(copy.set_build_history(history).is_err());
    assert_eq!(
        copy.to_bytes().unwrap(),
        project.to_bytes().unwrap(),
        "failed attachment must be atomic"
    );
}

#[test]
fn legacy_files_and_changed_content_do_not_get_a_stale_timeline() {
    let original = small_build();
    let bytes = original.to_bytes().unwrap();
    let mut project = from_bytes(&bytes).unwrap();
    let content_hash = project.semantic_hash().unwrap();
    let plain = Project::new(project.document.clone()).unwrap();
    assert_eq!(plain.semantic_hash().unwrap(), content_hash);
    let legacy = from_bytes(&plain.to_bytes().unwrap()).unwrap();
    let engine = legacy.open_engine(Limits::default()).unwrap();
    assert_eq!(engine.history().count(), 0);
    assert!(!engine.is_dirty());

    // The original template still contains the recipe. Saving changed content must
    // not merge it back in as unknown optional metadata.
    project.document.content.name = "Edited after opening".into();
    assert!(project.build_history().is_none());
    let saved = project.to_bytes().unwrap();
    assert!(
        !serde_json::from_slice::<Value>(&saved)
            .unwrap()
            .as_object()
            .unwrap()
            .contains_key("build_history")
    );
    let reopened = from_bytes(&saved).unwrap();
    assert_eq!(reopened.document, project.document);
    assert_eq!(
        reopened
            .open_engine(Limits::default())
            .unwrap()
            .history()
            .count(),
        0
    );
}

#[test]
fn replay_respects_host_limits_and_opens_clean() {
    let project = small_build();
    assert!(
        project
            .open_engine(Limits {
                history_bytes: 1,
                ..Limits::default()
            })
            .is_err()
    );
    assert!(
        project
            .open_engine(Limits {
                work_units: 1,
                ..Limits::default()
            })
            .is_err()
    );
    let engine = project.open_engine(Limits::default()).unwrap();
    assert_eq!(engine.document(), &project.document);
    assert_eq!(engine.history().count(), 3);
    assert!(!engine.is_dirty());
}

#[test]
fn build_replay_and_undo_preserve_unknown_project_extensions() {
    let original = small_build();
    let bytes = original.to_bytes().unwrap();
    let raw = r#"{ "schema_version":1,"encoding":"json","payload": [1e2,"\u0061"] }"#;
    let text = format!(
        "{{\"extensions\":{{\"org.absent\":{{\"state\":{raw}}}}},{}",
        &std::str::from_utf8(&bytes).unwrap()[1..]
    );
    let project = from_bytes(text.as_bytes()).unwrap();
    assert!(project.build_history().is_some());
    let mut engine = project.open_engine(Limits::default()).unwrap();
    while let Some(head) = engine.head() {
        assert_eq!(
            engine
                .document()
                .extensions
                .get("org.absent", "state")
                .unwrap()
                .raw_json(),
            raw
        );
        engine.undo(engine.revision(), head).unwrap();
    }
    assert_eq!(engine.document().extensions, project.document.extensions);
    assert!(
        std::str::from_utf8(&project.to_bytes().unwrap())
            .unwrap()
            .contains(raw)
    );
}
